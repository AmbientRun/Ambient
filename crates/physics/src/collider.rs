use std::{collections::HashMap, f32::consts::PI, fmt::Debug, ops::Deref, sync::Arc};

use ambient_core::{
    asset_cache,
    async_ecs::async_run,
    runtime,
    transform::{rotation, scale, translation},
};
use ambient_ecs::{
    components, query, Component, ComponentQuery, ComponentValueBase, Debuggable, Description, EntityData, EntityId, MakeDefault, Name,
    Networked, QueryEvent, QueryState, Store, SystemGroup, TypedReadQuery, World,
};
use ambient_editor_derive::ElementEditor;
use ambient_model::model_from_url;
use ambient_std::{
    asset_cache::{AssetCache, AsyncAssetKey, AsyncAssetKeyExt, SyncAssetKeyExt},
    asset_url::{AbsAssetUrl, ColliderAssetType, TypedAssetUrl},
    download_asset::{AssetError, JsonFromUrl},
    events::EventDispatcher,
};
use ambient_ui::Editable;
use anyhow::Context;
use async_trait::async_trait;
use futures::future::try_join_all;
use glam::{vec3, Mat4, Quat, Vec3};
use itertools::Itertools;
use physxx::{
    AsPxActor, AsPxRigidActor, PxActor, PxActorFlag, PxBase, PxBoxGeometry, PxControllerDesc, PxControllerShapeDesc, PxConvexMeshGeometry,
    PxGeometry, PxMaterial, PxMeshScale, PxPlaneGeometry, PxRigidActor, PxRigidBody, PxRigidBodyFlag, PxRigidDynamicRef, PxRigidStaticRef,
    PxShape, PxShapeFlag, PxSphereGeometry, PxTransform, PxTriangleMeshGeometry, PxUserData,
};
use serde::{Deserialize, Serialize};

use crate::{
    main_controller_manager, make_physics_static,
    mesh::{PhysxGeometry, PhysxGeometryFromUrl},
    physx::{
        angular_velocity, character_controller, contact_offset, linear_velocity, physics, physics_controlled, physics_shape, rest_offset,
        rigid_actor, Physics,
    },
    wood_physics_material, ColliderScene, PxActorUserData, PxShapeUserData, PxWoodMaterialKey,
};

fn one() -> f32 {
    1.0
}

components!("physics", {
    @[
        Debuggable, Networked, Store,
        Name["Plane collider"],
        Description["If attached, this entity will have a plane physics collider."]
    ]
    plane_collider: (),
    @[
        Debuggable, Networked, Store,
        Name["Box collider"],
        Description["If attached, this entity will have a box physics collider.\n`x, y, z` is the size of the box."]
    ]
    box_collider: Vec3,
    @[
        Debuggable, Networked, Store,
        Name["Sphere collider"],
        Description["If attached, this entity will have a sphere physics collider.\nThe value corresponds to the radius of the sphere."]
    ]
    sphere_collider: f32,
    @[
        Debuggable, Networked, Store,
        Name["Collider from URL"],
        Description["This entity will load its physics collider from the URL.\nThe value is the URL to load from."]
    ]
    collider_from_url: String,

    @[MakeDefault, Editable, Networked, Store]
    collider: ColliderDef,
    @[MakeDefault, Editable, Networked, Store]
    collider_type: ColliderType,
    collider_shapes: Vec<PxShape>,
    collider_shapes_convex: Vec<PxShape>,
    on_collider_loaded: EventDispatcher<dyn Fn(&mut World, EntityId) + Sync + Send>,

    @[
        Debuggable, Networked, Store,
        Name["Dynamic"],
        Description["If this is true, the entity will be dynamic (i.e. be able to move). Otherwise, it will be static."]
    ]
    dynamic: bool,
    @[
        Debuggable, MakeDefault[one], Editable, Networked, Store,
        Name["Mass"],
        Description["The mass of this entity, measured in kilograms."]
    ]
    mass: f32,
    @[
        MakeDefault[one], Editable, Networked, Store, Debuggable,
        Name["Density"],
        Description["The density of this entity.\nThis is used to update the `mass` when the entity is rescaled."]
    ]
    density: f32,

    @[
        Debuggable, MakeDefault, Networked, Store,
        Name["Character controller height"],
        Description["The height of the physics character controller attached to this entity.\nIf an entity has both this and a `character_controller_radius`, it will be given a physical character collider."]
    ]
    character_controller_height: f32,
    @[
        Debuggable, MakeDefault, Networked, Store,
        Name["Character controller radius"],
        Description["The radius of the physics character controller attached to this entity.\nIf an entity has both this and a `character_controller_height`, it will be given a physical character collider."]
    ]
    character_controller_radius: f32,
});

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ElementEditor)]
#[repr(usize)]
pub enum ColliderType {
    /// This object cannot move (e.g. a wall).
    Static,
    /// This object can move dynamically in the scene (e.g. a physics object).
    Dynamic,
    /// This object should only be present in the trigger-area scene.
    TriggerArea,
    /// This object should only be present in the picking scene.
    Picking,
}

impl ColliderType {
    pub fn scene(&self) -> ColliderScene {
        match self {
            ColliderType::Static => ColliderScene::Physics,
            ColliderType::Dynamic => ColliderScene::Physics,
            ColliderType::TriggerArea => ColliderScene::TriggerArea,
            ColliderType::Picking => ColliderScene::Picking,
        }
    }
}

impl Default for ColliderType {
    fn default() -> Self {
        Self::Static
    }
}

fn changed_or_missing<'a, T: ComponentValueBase, R: ComponentQuery<'a> + Clone + 'static>(
    q: &TypedReadQuery<R>,
    world: &'a World,
    qs: Option<&'a mut QueryState>,
    missing_component: Component<T>,
) -> Vec<(EntityId, <R as ComponentQuery<'a>>::DataCloned)> {
    let updated = q.collect_cloned(world, qs);
    let mut missing_q = q.clone();
    missing_q.query.event = QueryEvent::Frame;
    let missing = missing_q.excl(missing_component).iter_cloned(world, None).collect_vec();

    updated.into_iter().chain(missing.into_iter()).sorted_by_key(|x| x.0).dedup_by(|x, y| x.0 == y.0).collect_vec()
}

pub fn server_systems() -> SystemGroup {
    SystemGroup::new(
        "physics/collider/server",
        vec![
            query(plane_collider().changed()).to_system(|q, world, qs, _| {
                for (id, _) in changed_or_missing(q, world, qs, collider()) {
                    world.add_component(id, collider(), ColliderDef::Plane).unwrap();
                }
            }),
            query(sphere_collider().changed()).to_system(|q, world, qs, _| {
                for (id, radius) in changed_or_missing(q, world, qs, collider()) {
                    world.add_component(id, collider(), ColliderDef::Sphere { radius, center: Vec3::ZERO }).unwrap();
                }
            }),
            query(box_collider().changed()).to_system(|q, world, qs, _| {
                for (id, size) in changed_or_missing(q, world, qs, collider()) {
                    world.add_component(id, collider(), ColliderDef::Box { size, center: Vec3::ZERO }).unwrap();
                }
            }),
            query(collider_from_url().changed()).to_system(|q, world, qs, _| {
                for (id, url) in changed_or_missing(q, world, qs, collider()) {
                    match TypedAssetUrl::parse(&url) {
                        Ok(url) => world.add_component(id, collider(), ColliderDef::Asset { collider: url }).unwrap(),
                        Err(err) => log::warn!("Failed to load collider from {}: {:?}", url, err),
                    }
                }
            }),
            query(dynamic()).spawned().to_system(|q, world, qs, _| {
                for (id, dynamic) in changed_or_missing(q, world, qs, collider_type()) {
                    world.add_component(id, collider_type(), if dynamic { ColliderType::Dynamic } else { ColliderType::Static }).unwrap();
                }
            }),
            query((character_controller_height().changed(), character_controller_radius().changed(), translation())).to_system(
                |q, world, qs, _| {
                    let all = changed_or_missing(q, world, qs, character_controller());

                    for (id, (height, radius, pos)) in all {
                        if let Ok(old) = world.get(id, character_controller()) {
                            old.release();
                        }
                        let controller_manager = world.resource(main_controller_manager());
                        let physics_material = world.resource(wood_physics_material()).clone();

                        let mut desc = PxControllerDesc::new(
                            PxControllerShapeDesc::Capsule { radius, height: height - radius * 2. },
                            physics_material,
                        );
                        if desc.is_valid() {
                            desc.position = pos.as_dvec3();
                            desc.up_direction = vec3(0., 0., 1.);
                            let controller = controller_manager.create_controller(&desc);
                            for shape in controller.get_actor().get_shapes() {
                                shape.set_flag(PxShapeFlag::VISUALIZATION, false);
                            }
                            let actor = controller.get_actor();
                            actor.as_actor().set_user_data(id);
                            actor.get_shapes()[0].set_user_data(PxShapeUserData { entity: id, density: 1., ..Default::default() });
                            world.add_component(id, character_controller(), controller).unwrap();
                        } else {
                            world.remove_component(id, character_controller()).unwrap();
                        }
                    }
                },
            ),
            query((collider().changed(),)).optional_changed(model_from_url()).optional_changed(density()).to_system(|q, world, qs, _| {
                let all = changed_or_missing(q, world, qs, collider_shapes());

                let mut by_collider = HashMap::new();
                for (id, (collider_def,)) in all {
                    let density = world.get(id, density()).unwrap_or(1.);
                    let entry =
                        by_collider.entry(format!("{collider_def:?}-{density}")).or_insert_with(|| (collider_def, density, Vec::new()));
                    entry.2.push(id);
                }
                if by_collider.is_empty() {
                    return;
                }
                let assets = world.resource(asset_cache()).clone();
                let runtime = world.resource(runtime()).clone();
                let async_run = world.resource(async_run()).clone();
                for (collider_def, density, mut ids) in by_collider.into_values() {
                    let assets = assets.clone();
                    let async_run = async_run.clone();
                    runtime.spawn(async move {
                        let collider_spawner = match collider_def.spawner(assets, density).await {
                            Ok(collider_spawner) => collider_spawner,
                            Err(err) => {
                                tracing::warn!("Failed to load collider: {:#}", err);
                                return;
                            }
                        };
                        async_run.run(move |world| {
                            ids.retain(|id| world.exists(*id));
                            for id in ids.into_iter() {
                                let physics = world.resource(physics());
                                let (shapes, convex) = (collider_spawner)(physics, world.get(id, scale()).unwrap_or(Vec3::ONE));
                                if !shapes.is_empty() {
                                    for shape in &shapes {
                                        shape.set_flag(PxShapeFlag::SCENE_QUERY_SHAPE, true);
                                        shape.set_flag(PxShapeFlag::VISUALIZATION, false);
                                    }

                                    for shape in &convex {
                                        shape.set_flag(PxShapeFlag::SCENE_QUERY_SHAPE, true);
                                        shape.set_flag(PxShapeFlag::VISUALIZATION, false);
                                    }

                                    world.add_component(id, collider_shapes(), shapes).unwrap();
                                    world.add_component(id, collider_shapes_convex(), convex).unwrap();
                                }
                            }
                        });
                    });
                }
            }),
            query(collider_shapes().changed()).optional_changed(collider_type()).to_system(|q, world, qs, _| {
                let physics = world.resource(physics()).clone();
                let force_static = world.get(world.resource_entity(), make_physics_static()).unwrap_or(false);
                let build_actor = |world: &mut World, id: EntityId, mut shapes: Vec<PxShape>| {
                    let pos = world.get(id, translation()).unwrap_or_default();
                    let rot = world.get(id, rotation()).unwrap_or_default();
                    let collider_type = world.get(id, collider_type()).unwrap_or(ColliderType::Static);
                    if let Ok(actor) = world.get(id, rigid_actor()) {
                        if let Some(scene) = actor.get_scene() {
                            scene.remove_actor(&actor, false);
                        }
                    }
                    let actor = if collider_type == ColliderType::Dynamic && !force_static {
                        world.add_component(id, physics_controlled(), ()).unwrap();
                        let lvel = world.get(id, linear_velocity()).unwrap_or_default();
                        let avel = world.get(id, angular_velocity()).unwrap_or_default();
                        let body = PxRigidDynamicRef::new(physics.physics, &PxTransform::new(pos, rot));
                        body.set_linear_velocity(lvel, true);
                        body.set_angular_velocity(avel, true);
                        body.as_rigid_actor()
                    } else {
                        world.remove_component(id, physics_controlled()).unwrap();
                        PxRigidStaticRef::new(physics.physics, &PxTransform::new(pos, rot)).as_rigid_actor()
                    };
                    if let Some(actor) = actor.to_rigid_body() {
                        actor.set_rigid_body_flag(PxRigidBodyFlag::ENABLE_CCD, true);
                    }
                    actor.as_actor().set_user_data(PxActorUserData { serialize: true });
                    for shape in actor.get_shapes() {
                        actor.detach_shape(&shape, false);
                    }
                    let coff = world.get(id, contact_offset()).ok();
                    let roff = world.get(id, rest_offset()).ok();
                    for shape in shapes.iter_mut() {
                        if !actor.attach_shape(shape) {
                            log::error!("Failed to attach shape to entity {}", id);
                            actor.as_actor().remove_user_data::<PxActorUserData>();
                            actor.release();
                            return;
                        }
                        // TODO(josh): shapes should probably have their own ECS objects
                        if let Some(coff) = coff {
                            shape.set_contact_offset(coff);
                        }
                        if let Some(roff) = roff {
                            shape.set_rest_offset(roff);
                        }
                        shape.update_user_data::<PxShapeUserData>(&|ud| ud.entity = id);
                    }
                    if let Some(actor) = actor.to_rigid_dynamic() {
                        let densities =
                            actor.get_shapes().iter().map(|shape| shape.get_user_data::<PxShapeUserData>().unwrap().density).collect_vec();
                        actor.update_mass_and_inertia(densities, None, None);
                        world.add_component(id, mass(), actor.get_mass()).unwrap();
                    } else {
                        world.remove_component(id, mass()).ok();
                    }
                    let first_shape = shapes[0].clone();
                    world.add_components(id, EntityData::new().set(physics_shape(), first_shape).set(rigid_actor(), actor)).unwrap();
                    actor.set_actor_flag(PxActorFlag::VISUALIZATION, false);
                    if collider_type != ColliderType::Dynamic && collider_type != ColliderType::Static {
                        actor.set_actor_flag(PxActorFlag::DISABLE_SIMULATION, true);
                    }
                    let scene = collider_type.scene().get_scene(world);
                    scene.add_actor(&actor);
                    world.resource_mut(crate::collider_loads()).push(id);
                    if let Ok(event) = world.get_ref(id, on_collider_loaded()).cloned() {
                        for handler in event.iter() {
                            (*handler)(world, id);
                        }
                    }
                };
                for (id, shapes) in q.collect_cloned(world, qs) {
                    build_actor(world, id, shapes);
                }
            }),
        ],
    )
}

fn one_value() -> f32 {
    1.
}
fn vec3_zero_value() -> Vec3 {
    Vec3::ZERO
}

#[derive(Serialize, Deserialize, Debug, Clone, ElementEditor)]
#[editor(prompt = "Collider Definition")]
pub enum ColliderDef {
    Asset {
        collider: TypedAssetUrl<ColliderAssetType>,
    },
    Box {
        size: Vec3,
        #[serde(default = "vec3_zero_value")]
        center: Vec3,
    },
    Sphere {
        #[serde(default = "one_value")]
        radius: f32,
        #[serde(default = "vec3_zero_value")]
        center: Vec3,
    },
    Plane,
}

type ColliderSpawner = Box<dyn Fn(&Physics, Vec3) -> (Vec<PxShape>, Vec<PxShape>) + Sync + Send>;
impl ColliderDef {
    #[allow(clippy::single_match)]
    pub fn resolve(&mut self, base_url: &AbsAssetUrl) -> anyhow::Result<()> {
        match self {
            ColliderDef::Asset { collider } => {
                *collider = collider.resolve(base_url).context("Failed to resolve")?.into();
            }
            _ => {}
        }
        Ok(())
    }

    /// Generate a closure which will spawn a shape into the world given the in-world scale.
    ///
    /// **Note**: this scale is applied after the initial base_pose scale.
    pub async fn spawner(&self, assets: AssetCache, density: f32) -> Result<ColliderSpawner, AssetError> {
        let material = PxWoodMaterialKey.get(&assets);
        match self.clone() {
            ColliderDef::Box { size, center } => Ok(Box::new(move |physics, scale| {
                let size = size * scale;
                let geometry = PxBoxGeometry::new(size.x / 2., size.y / 2., size.x / 2.);
                let shape = PxShape::new(physics.physics, &geometry, &[&material], Some(true), None);
                shape.set_local_pose(&PxTransform::from_translation(center * scale));
                shape.set_user_data(PxShapeUserData {
                    entity: EntityId::null(),
                    density,
                    base_pose: Mat4::from_scale_rotation_translation(size / scale, Default::default(), center * scale),
                });
                (vec![shape.clone()], vec![shape])
            })),
            ColliderDef::Sphere { radius, center } => Ok(Box::new(move |physics, scale| {
                if radius <= 0. {
                    return (vec![], vec![]);
                }
                let geometry = PxSphereGeometry::new(radius * scale.x);
                let shape = PxShape::new(physics.physics, &geometry, &[&material], Some(true), None);
                shape.set_local_pose(&PxTransform::from_translation(center * scale));
                shape.set_user_data(PxShapeUserData {
                    entity: EntityId::null(),
                    density,
                    base_pose: Mat4::from_scale_rotation_translation(Vec3::splat(radius), Default::default(), center * scale),
                });
                (vec![shape.clone()], vec![shape])
            })),
            ColliderDef::Plane => Ok(Box::new(move |physics, _scale| {
                let geometry = PxPlaneGeometry::new();
                let shape = PxShape::new(physics.physics, &geometry, &[&material], Some(true), None);
                shape.set_local_pose(&PxTransform::from_rotation(Quat::from_rotation_y(-PI / 2.)));
                shape.set_user_data(PxShapeUserData { entity: EntityId::null(), density, base_pose: Mat4::from_rotation_y(-PI / 2.) });
                (vec![shape.clone()], vec![shape])
            })),
            ColliderDef::Asset { collider } => {
                let collider = collider.unwrap_abs();
                let collider_from_urls: Arc<ColliderFromUrls> = JsonFromUrl::new(collider.clone(), true).get(&assets).await?;
                let collider = collider_from_urls.resolve(&collider)?.get(&assets).await?;

                Ok(Box::new(move |physics, scale| {
                    (
                        collider.spawn(physics, scale, material.clone(), density, false),
                        collider.spawn(physics, scale, material.clone(), density, true),
                    )
                }))
            }
        }
    }
}

impl Default for ColliderDef {
    fn default() -> Self {
        Self::Sphere { radius: 1., center: Vec3::ZERO }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColliderFromUrls {
    pub concave: Vec<(Mat4, PhysxGeometryFromUrl)>,
    pub convex: Vec<(Mat4, PhysxGeometryFromUrl)>,
}
impl ColliderFromUrls {
    pub fn resolve(&self, base_url: &AbsAssetUrl) -> anyhow::Result<Self> {
        Ok(Self {
            concave: self.concave.iter().map(|(mat, url)| Ok((*mat, url.resolve(base_url)?))).collect::<anyhow::Result<Vec<_>>>()?,
            convex: self.convex.iter().map(|(mat, url)| Ok((*mat, url.resolve(base_url)?))).collect::<anyhow::Result<Vec<_>>>()?,
        })
    }
}

#[async_trait]
impl AsyncAssetKey<Result<Arc<Collider>, AssetError>> for ColliderFromUrls {
    async fn load(self, assets: AssetCache) -> Result<Arc<Collider>, AssetError> {
        let mut res: Vec<_> = try_join_all([self.concave, self.convex].into_iter().map(|list| async {
            let iter = list.into_iter().map(|(transform, mesh)| {
                let assets = assets.clone();
                async move { Ok::<_, AssetError>((transform, mesh.get(&assets).await?.deref().clone())) }
            });

            let colliders: Vec<_> = try_join_all(iter).await?;
            Ok(colliders) as Result<_, AssetError>
        }))
        .await?;
        let convex = res.pop().unwrap();
        let concave = res.pop().unwrap();

        Ok(Arc::new(Collider { convex, concave }))
    }
}

/// A collider is a collection of gemoetries. It's also got a convex version used for for instance object placement
#[derive(Debug, Clone)]
struct Collider {
    pub concave: Vec<(Mat4, PhysxGeometry)>,
    pub convex: Vec<(Mat4, PhysxGeometry)>,
}

impl Collider {
    pub fn spawn(&self, physics: &Physics, scale: Vec3, material: PxMaterial, density: f32, convex: bool) -> Vec<PxShape> {
        if convex { &self.convex } else { &self.concave }
            .iter()
            .map(|(transform, mesh)| {
                tracing::info!(transform = ?transform.to_scale_rotation_translation(), "Spawning convex mesh with transform");
                // Read the scale that was applied with the model local transform
                let (scale, rotation, translation) = (Mat4::from_scale(scale) * *transform).to_scale_rotation_translation();

                let geometry: Box<dyn PxGeometry> = match mesh {
                    PhysxGeometry::ConvexMesh(mesh) => {
                        let geometry = PxConvexMeshGeometry::new(mesh, Some(PxMeshScale::from_scale(scale.abs())), None);
                        if !geometry.is_valid() {
                            panic!("Invalid geometry. scale={scale:?}");
                        }
                        Box::new(geometry)
                    }
                    PhysxGeometry::TriangleMesh(mesh) => {
                        let geometry = PxTriangleMeshGeometry::new(mesh, Some(PxMeshScale::from_scale(scale)), None);
                        if !geometry.is_valid() {
                            panic!("Invalid geometry. scale={scale:?}");
                        }
                        Box::new(geometry)
                    }
                };
                let shape = PxShape::new(physics.physics, &*geometry, &[&material], Some(true), None);

                shape.set_local_pose(&PxTransform::new(scale * translation, rotation));
                shape.set_user_data(PxShapeUserData {
                    entity: EntityId::null(),
                    density,
                    base_pose: Mat4::from_scale_rotation_translation(scale.abs(), rotation, translation),
                });
                shape
            })
            .collect_vec()
    }
}
