use std::collections::HashSet;

use elements_ecs::{
    components, ensure_has_component, query, query_mut, Debuggable, Description, ECSError, EntityId, FrameEvent, MakeDefault, Name, Networked, QueryState, Store, System, SystemGroup, World
};
use glam::*;

use crate::{
    camera::get_active_camera, gpu_components, gpu_ecs::{ComponentToGpuSystem, GpuComponentFormat, GpuWorldSyncEvent}, hierarchy::{children, parent}, main_scene
};

fn vec3_one() -> Vec3 {
    Vec3::ONE
}

components!("transform", {
    @[
        MakeDefault, Debuggable, Networked, Store,
        Name["Translation"],
        Description["The translation/position of this entity"]
    ]
    translation: Vec3,
    @[
        MakeDefault[vec3_one], Debuggable, Networked, Store,
        Name["Scale"],
        Description["The scale of this entity"]
    ]
    scale: Vec3,
    @[
        Debuggable, Networked, Store,
        Name["Rotation"],
        Description["The rotation of this entity"]
    ]
    rotation: Quat,
    @[
        MakeDefault, Debuggable, Networked, Store,
        Name["Euler rotation"],
        Description["The Euler rotation of this entity in ZYX order."]
    ]
    euler_rotation: Vec3,
    @[
        Debuggable, Networked, Store,
        Name["Look-at center"],
        Description["The position that this entity should be looking at"]
    ]
    lookat_center: Vec3,
    @[
        Debuggable, Networked, Store,
        Name["Look-at up"],
        Description["When combined with `lookat_center`, the up vector for this entity"]
    ]
    lookat_up: Vec3,

    @[
        Debuggable, Networked, Store,
        Name["Local to World"],
        Description["Transformation from the entity's local space to worldspace"]
    ]
    local_to_world: Mat4,
    @[
        Debuggable, Networked, Store,
        Name["Inverse Local to World"],
        Description["Converts a world position to a local position; driven by `local_to_world`"]
    ]
    inv_local_to_world: Mat4,

    @[
        Debuggable, Networked, Store,
        Name["Local to Parent"],
        Description["Transformation from the entity's local space to the parent's space"]
    ]
    local_to_parent: Mat4,
    @[
        Debuggable, Networked, Store,
        Name["Mesh to Local"],
        Description["Transformation from mesh-space to the entity's local space."]
    ]
    mesh_to_local: Mat4,
    @[
        Debuggable, Networked, Store,
        Name["Mesh to World"],
        Description["Transformation from mesh-space to world space; driven by `mesh_to_local` and `local_to_world`"]
    ]
    mesh_to_world: Mat4,
    @[
        Debuggable, Networked, Store,
        Name["Spherical billboard"],
        Description["Ensures that this entity is always aligned with the camera"]
    ]
    spherical_billboard: (),
    @[
        Debuggable, Networked, Store,
        Name["Cylindrical billboard Z"],
        Description["Ensures this entity is always aligned with the camera, except on the Z-axis; useful for trees and such"]
    ]
    cylindrical_billboard_z: (),

    @[
        Debuggable, Networked, Store,
        Name["Reset scale"],
        Description["When this component is applied to a transform hierarchy, the scale will be reset at that point (and only rotation/translation considered)"]
    ]
    reset_scale: (),

    // FBX
    @[Debuggable, Networked, Store]
    fbx_complex_transform: (),
    @[Debuggable, Networked, Store]
    fbx_rotation_offset: Vec3,
    @[Debuggable, Networked, Store]
    fbx_rotation_pivot: Vec3,
    @[Debuggable, Networked, Store]
    fbx_pre_rotation: Quat,
    @[Debuggable, Networked, Store]
    fbx_post_rotation: Quat,
    @[Debuggable, Networked, Store]
    fbx_scaling_offset: Vec3,
    @[Debuggable, Networked, Store]
    fbx_scaling_pivot: Vec3,
});

gpu_components! {
    mesh_to_world() => mesh_to_world: GpuComponentFormat::Mat4,
}

#[derive(Debug)]
pub struct TransformSystem {
    systems: SystemGroup,
    post_parented_systems: SystemGroup,
    parented_state_1: QueryState,
    parented_state_2: QueryState,
}
impl TransformSystem {
    pub fn new() -> Self {
        Self {
            systems: SystemGroup::new(
                "transform_systems",
                vec![
                    query_mut((rotation(),), (euler_rotation().changed(),)).to_system(|query, world, state, _| {
                        for (_, (rot,), (&r,)) in query.iter(world, state) {
                            *rot = Quat::from_euler(EulerRot::ZYX, r.z, r.y, r.x);
                        }
                    }),
                    query_mut((local_to_parent(),), (translation().changed(), rotation().changed(), scale().changed()))
                        .excl(fbx_complex_transform())
                        .to_system(|query, world, state, _| {
                            for (_, (local_to_parent,), (&translation, &rotation, &scale)) in query.iter(world, state) {
                                *local_to_parent = Mat4::from_scale_rotation_translation(scale, rotation, translation);
                            }
                        }),
                    query_mut((local_to_parent(),), (translation().changed(), scale().changed()))
                        .excl(rotation())
                        .excl(fbx_complex_transform())
                        .to_system(|query, world, state, _| {
                            for (_, (local_to_parent,), (&translation, &scale)) in query.iter(world, state) {
                                *local_to_parent = Mat4::from_scale_rotation_translation(scale, Quat::IDENTITY, translation);
                            }
                        }),
                    query_mut((local_to_parent(),), (translation().changed(), rotation().changed()))
                        .excl(scale())
                        .excl(fbx_complex_transform())
                        .to_system(|query, world, state, _| {
                            for (_, (local_to_parent,), (&translation, &rotation)) in query.iter(world, state) {
                                *local_to_parent = Mat4::from_rotation_translation(rotation, translation);
                            }
                        }),
                    query_mut((local_to_parent(),), (scale().changed(), rotation().changed()))
                        .excl(translation())
                        .excl(fbx_complex_transform())
                        .to_system(|query, world, state, _| {
                            for (_, (local_to_parent,), (&scale, &rotation)) in query.iter(world, state) {
                                *local_to_parent = Mat4::from_scale_rotation_translation(scale, rotation, Vec3::ZERO);
                            }
                        }),
                    query_mut((local_to_parent(),), (translation().changed(),))
                        .excl(scale())
                        .excl(rotation())
                        .excl(fbx_complex_transform())
                        .to_system(|query, world, state, _| {
                            for (_, (local_to_parent,), (&translation,)) in query.iter(world, state) {
                                *local_to_parent = Mat4::from_translation(translation);
                            }
                        }),
                    query_mut((local_to_parent(),), (rotation().changed(),))
                        .excl(scale())
                        .excl(translation())
                        .excl(fbx_complex_transform())
                        .to_system(|query, world, state, _| {
                            for (_, (local_to_parent,), (&rotation,)) in query.iter(world, state) {
                                *local_to_parent = Mat4::from_quat(rotation);
                            }
                        }),
                    query_mut((local_to_parent(),), (scale().changed(),))
                        .excl(rotation())
                        .excl(translation())
                        .excl(fbx_complex_transform())
                        .to_system(|query, world, state, _| {
                            for (_, (local_to_parent,), (&scale,)) in query.iter(world, state) {
                                *local_to_parent = Mat4::from_scale(scale);
                            }
                        }),
                    query_mut((local_to_world(),), (translation().changed(), rotation().changed(), scale().changed()))
                        .excl(local_to_parent())
                        .excl(lookat_center())
                        .excl(fbx_complex_transform())
                        .to_system(|query, world, state, _| {
                            for (_, (local_to_world,), (&translation, &rotation, &scale)) in query.iter(world, state) {
                                *local_to_world = Mat4::from_scale_rotation_translation(scale, rotation, translation);
                            }
                        }),
                    query_mut((local_to_world(),), (translation().changed(), rotation().changed()))
                        .excl(local_to_parent())
                        .excl(lookat_center())
                        .excl(scale())
                        .excl(fbx_complex_transform())
                        .to_system(|q, world, qs, _| {
                            for (_, (local_to_world,), (&translation, &rotation)) in q.iter(world, qs) {
                                *local_to_world = Mat4::from_rotation_translation(rotation, translation);
                            }
                        }),
                    query_mut((local_to_world(),), (translation().changed(), scale().changed()))
                        .excl(local_to_parent())
                        .excl(lookat_center())
                        .excl(rotation())
                        .excl(fbx_complex_transform())
                        .to_system(|q, world, qs, _| {
                            for (_, (local_to_world,), (&translation, &scale)) in q.iter(world, qs) {
                                *local_to_world = Mat4::from_scale_rotation_translation(scale, Quat::IDENTITY, translation);
                            }
                        }),
                    query_mut((local_to_world(),), (rotation().changed(), scale().changed()))
                        .excl(local_to_parent())
                        .excl(lookat_center())
                        .excl(translation())
                        .excl(fbx_complex_transform())
                        .to_system(|q, world, qs, _| {
                            for (_, (local_to_world,), (&rotation, &scale)) in q.iter(world, qs) {
                                *local_to_world = Mat4::from_scale_rotation_translation(scale, rotation, Vec3::ZERO);
                            }
                        }),
                    query_mut((local_to_world(),), (translation().changed(),))
                        .excl(local_to_parent())
                        .excl(lookat_center())
                        .excl(scale())
                        .excl(rotation())
                        .excl(fbx_complex_transform())
                        .to_system(|q, world, qs, _| {
                            for (_, (local_to_world,), (&translation,)) in q.iter(world, qs) {
                                *local_to_world = Mat4::from_translation(translation);
                            }
                        }),
                    query_mut((local_to_world(),), (scale().changed(),))
                        .excl(local_to_parent())
                        .excl(lookat_center())
                        .excl(translation())
                        .excl(rotation())
                        .excl(fbx_complex_transform())
                        .to_system(|q, world, qs, _| {
                            for (_, (local_to_world,), (&scale,)) in q.iter(world, qs) {
                                *local_to_world = Mat4::from_scale(scale);
                            }
                        }),
                    query_mut((local_to_world(),), (rotation().changed(),))
                        .excl(local_to_parent())
                        .excl(lookat_center())
                        .excl(translation())
                        .excl(scale())
                        .excl(fbx_complex_transform())
                        .to_system(|q, world, qs, _| {
                            for (_, (local_to_world,), (&rotation,)) in q.iter(world, qs) {
                                *local_to_world = Mat4::from_quat(rotation);
                            }
                        }),
                    // Make sure lookat has all the components
                    ensure_has_component(lookat_center(), local_to_world(), Default::default()),
                    ensure_has_component(lookat_center(), inv_local_to_world(), Default::default()),
                    ensure_has_component(lookat_center(), translation(), Default::default()),
                    ensure_has_component(lookat_center(), lookat_up(), Vec3::Z),
                    query_mut(
                        (local_to_world(), inv_local_to_world()),
                        (translation().changed(), lookat_center().changed(), lookat_up().changed()),
                    )
                    .excl(local_to_parent())
                    .excl(fbx_complex_transform())
                    .to_system(|q, world, qs, _| {
                        for (_, (local_to_world, inv_local_to_world), (&translation, &lookat_center, &lookat_up)) in q.iter(world, qs) {
                            *inv_local_to_world = Mat4::look_at_lh(translation, lookat_center, lookat_up);
                            *local_to_world = inv_local_to_world.inverse();
                        }
                    }),
                    // FBX
                    query((fbx_complex_transform(), local_to_parent()))
                        .optional_changed(translation())
                        .optional_changed(fbx_rotation_offset())
                        .optional_changed(fbx_rotation_pivot())
                        .optional_changed(fbx_pre_rotation())
                        .optional_changed(rotation())
                        .optional_changed(fbx_post_rotation())
                        .optional_changed(fbx_rotation_pivot())
                        .optional_changed(fbx_scaling_offset())
                        .optional_changed(fbx_scaling_pivot())
                        .optional_changed(scale())
                        .optional_changed(fbx_scaling_pivot())
                        .to_system(|q, world, qs, _| {
                            // See: https://help.autodesk.com/view/FBX/2017/ENU/?guid=__files_GUID_10CDD63C_79C1_4F2D_BB28_AD2BE65A02ED_htm
                            // and: https://github.com/assimp/assimp/blob/add7f1355e96c6ff0df0ba3cec084f25332d154e/code/AssetLib/FBX/FBXConverter.cpp#L687
                            for (id, _) in q.collect_cloned(world, qs) {
                                world.set(id, local_to_parent(), get_fbx_transform(world, id)).unwrap();
                            }
                        }),
                    query((fbx_complex_transform(), local_to_world()))
                        .optional_changed(translation())
                        .optional_changed(fbx_rotation_offset())
                        .optional_changed(fbx_rotation_pivot())
                        .optional_changed(fbx_pre_rotation())
                        .optional_changed(rotation())
                        .optional_changed(fbx_post_rotation())
                        .optional_changed(fbx_rotation_pivot())
                        .optional_changed(fbx_scaling_offset())
                        .optional_changed(fbx_scaling_pivot())
                        .optional_changed(scale())
                        .optional_changed(fbx_scaling_pivot())
                        .to_system(|q, world, qs, _| {
                            // See: https://help.autodesk.com/view/FBX/2017/ENU/?guid=__files_GUID_10CDD63C_79C1_4F2D_BB28_AD2BE65A02ED_htm
                            // and: https://github.com/assimp/assimp/blob/add7f1355e96c6ff0df0ba3cec084f25332d154e/code/AssetLib/FBX/FBXConverter.cpp#L687
                            for (id, _) in q.collect_cloned(world, qs) {
                                world.set(id, local_to_world(), get_fbx_transform(world, id)).unwrap();
                            }
                        }),
                ],
            ),
            post_parented_systems: SystemGroup::new(
                "transform_systems",
                vec![
                    query_mut((mesh_to_world(),), (local_to_world().changed(), mesh_to_local().changed())).to_system(|q, world, qs, _| {
                        for (_, (mesh_to_world,), (&local_to_world, &mesh_to_local)) in q.iter(world, qs) {
                            *mesh_to_world = local_to_world * mesh_to_local;
                        }
                    }),
                    query_mut((mesh_to_world(),), (local_to_world().changed(),)).excl(mesh_to_local()).to_system(|q, world, qs, _| {
                        for (_, (mesh_to_world,), (&local_to_world,)) in q.iter(world, qs) {
                            *mesh_to_world = local_to_world;
                        }
                    }),
                    query_mut((inv_local_to_world(),), (local_to_world().changed(),)).excl(lookat_center()).to_system(|q, world, qs, _| {
                        for (_, (inv_local_to_world,), (local_to_world,)) in q.iter(world, qs) {
                            *inv_local_to_world = local_to_world.inverse();
                        }
                    }),
                ],
            ),
            parented_state_1: QueryState::new(),
            parented_state_2: QueryState::new(),
        }
    }

    #[profiling::function]
    fn parented(&mut self, world: &mut World) {
        let mut changed_roots = HashSet::<EntityId>::new();
        for (id, _) in query((local_to_parent().changed(),)).iter(world, Some(&mut self.parented_state_1)) {
            // TODO: This could be optimized
            changed_roots.insert(get_transform_root(world, id));
        }
        for (id, (), (_, _)) in
            query_mut((), (local_to_world().changed(), children())).excl(local_to_parent()).iter(world, Some(&mut self.parented_state_2))
        {
            changed_roots.insert(id);
        }

        for id in changed_roots.into_iter() {
            if let Ok(transform) = world.get(id, local_to_parent()) {
                if world.set(id, local_to_world(), transform).is_err() {
                    return;
                }
                if let Ok(children) = world.get_ref(id, children()).cloned() {
                    for child in children {
                        update_transform_recursive(world, child, transform);
                    }
                }
            } else if let Ok(transform) = world.get(id, local_to_world()) {
                if let Ok(children) = world.get_ref(id, children()).cloned() {
                    for child in children {
                        update_transform_recursive(world, child, transform);
                    }
                }
            } else {
                log::warn!("Bad transform hierarchy; bad root: {}", id);
            }
        }
    }
}
impl System for TransformSystem {
    fn run(&mut self, world: &mut World, event: &FrameEvent) {
        profiling::scope!("TransformSystem::run");
        self.systems.run(world, event);
        if let Some(camera) = get_active_camera(world, main_scene()) {
            let inv_view = world.get(camera, local_to_world()).ok();

            if let Some(inv_view) = inv_view {
                for (_, (local_to_world,), ()) in
                    query_mut((local_to_world(),), ()).excl(local_to_parent()).incl(spherical_billboard()).iter(world, None)
                {
                    spherical_billboard_matrix(local_to_world, &inv_view);
                }
                for (_, (local_to_world,), ()) in
                    query_mut((local_to_world(),), ()).excl(local_to_parent()).incl(cylindrical_billboard_z()).iter(world, None)
                {
                    cylindrical_billboard_z_matrix(local_to_world, &inv_view);
                }
            }
        }

        self.parented(world);
        self.post_parented_systems.run(world, event);
    }
}
pub fn transform_gpu_systems() -> SystemGroup<GpuWorldSyncEvent> {
    SystemGroup::new(
        "transform_gpu",
        vec![Box::new(ComponentToGpuSystem::new(GpuComponentFormat::Mat4, mesh_to_world(), gpu_components::mesh_to_world()))],
    )
}
fn update_transform_recursive(world: &mut World, id: EntityId, mut parent_transform: Mat4) {
    if world.has_component(id, reset_scale()) {
        let (_s, r, t) = parent_transform.to_scale_rotation_translation();
        parent_transform = Mat4::from_rotation_translation(r, t);
    }
    let transform = if let Ok(local_to_parent) = world.get(id, local_to_parent()) {
        parent_transform * local_to_parent
    } else {
        return;
    };
    if world.set(id, local_to_world(), transform).is_err() {
        return;
    }
    if let Ok(children) = world.get_ref(id, children()).cloned() {
        for child in children {
            update_transform_recursive(world, child, transform);
        }
    }
}
fn get_fbx_transform(world: &World, id: EntityId) -> Mat4 {
    world.get(id, translation()).map(Mat4::from_translation).unwrap_or_default()
        * world.get(id, fbx_rotation_offset()).map(Mat4::from_translation).unwrap_or_default()
        * world.get(id, fbx_rotation_pivot()).map(Mat4::from_translation).unwrap_or_default()
        * world.get(id, fbx_pre_rotation()).map(|v| Mat4::from_quat(v)).unwrap_or_default()
        * world.get(id, rotation()).map(|v| Mat4::from_quat(v)).unwrap_or_default()
        * world.get(id, fbx_post_rotation()).map(|v| Mat4::from_quat(v).inverse()).unwrap_or_default()
        * world.get(id, fbx_rotation_pivot()).map(|x| Mat4::from_translation(x).inverse()).unwrap_or_default()
        * world.get(id, fbx_scaling_offset()).map(Mat4::from_translation).unwrap_or_default()
        * world.get(id, fbx_scaling_pivot()).map(Mat4::from_translation).unwrap_or_default()
        * world.get(id, scale()).map(Mat4::from_scale).unwrap_or_default()
        * world.get(id, fbx_scaling_pivot()).map(|x| Mat4::from_translation(x).inverse()).unwrap_or_default()
}

fn get_transform_root(world: &World, id: EntityId) -> EntityId {
    if let Ok(parent) = world.get_ref(id, parent()) {
        if world.has_component(id, local_to_parent()) && world.has_component(*parent, local_to_world()) {
            return get_transform_root(world, *parent);
        }
    }
    id
}

fn spherical_billboard_matrix(local_to_world: &mut Mat4, inv_view: &Mat4) {
    local_to_world.as_mut()[0] = inv_view.as_ref()[0];
    local_to_world.as_mut()[1] = inv_view.as_ref()[1];
    local_to_world.as_mut()[2] = inv_view.as_ref()[2];

    local_to_world.as_mut()[4] = inv_view.as_ref()[4];
    local_to_world.as_mut()[4 + 1] = inv_view.as_ref()[4 + 1];
    local_to_world.as_mut()[4 + 2] = inv_view.as_ref()[4 + 2];

    local_to_world.as_mut()[2 * 4] = inv_view.as_ref()[2 * 4];
    local_to_world.as_mut()[2 * 4 + 1] = inv_view.as_ref()[2 * 4 + 1];
    local_to_world.as_mut()[2 * 4 + 2] = inv_view.as_ref()[2 * 4 + 2];
}
fn cylindrical_billboard_z_matrix(local_to_world: &mut Mat4, inv_view: &Mat4) {
    local_to_world.as_mut()[0] = inv_view.as_ref()[0];
    local_to_world.as_mut()[1] = inv_view.as_ref()[1];
    // local_to_world.as_mut()[2] = inv_view.as_ref()[2];

    local_to_world.as_mut()[4] = inv_view.as_ref()[4];
    local_to_world.as_mut()[4 + 1] = inv_view.as_ref()[4 + 1];
    // local_to_world.as_mut()[1*4 + 2] = inv_view.as_ref()[1*4 + 2];

    local_to_world.as_mut()[2 * 4] = inv_view.as_ref()[2 * 4];
    local_to_world.as_mut()[2 * 4 + 1] = inv_view.as_ref()[2 * 4 + 1];
    // local_to_world.as_mut()[2*4 + 2] = inv_view.as_ref()[2*4 + 2];
}

pub fn get_world_transform(world: &World, entity: EntityId) -> Result<Mat4, ECSError> {
    match world.get(entity, local_to_world()) {
        Ok(ltw) => Ok(ltw),
        Err(err) => match err {
            ECSError::EntityDoesntHaveComponent { .. } => Ok(Mat4::from_scale_rotation_translation(
                world.get(entity, scale()).unwrap_or(Vec3::ONE),
                world.get(entity, rotation()).unwrap_or(Quat::IDENTITY),
                world.get(entity, translation()).unwrap_or(Vec3::ZERO),
            )),
            ECSError::NoSuchEntity { .. } => Err(err),
        },
    }
}

pub fn get_world_position(world: &World, entity: EntityId) -> Result<Vec3, ECSError> {
    match world.get(entity, local_to_world()) {
        Ok(ltw) => Ok(ltw.transform_point3(Vec3::ZERO)),
        Err(err) => match err {
            ECSError::EntityDoesntHaveComponent { .. } => world.get(entity, translation()),
            ECSError::NoSuchEntity { .. } => Err(err),
        },
    }
}

pub fn get_world_rotation(world: &World, entity: EntityId) -> Result<Quat, ECSError> {
    match world.get(entity, local_to_world()) {
        Ok(ltw) => {
            let (_, rot, _) = ltw.to_scale_rotation_translation();
            Ok(rot)
        }
        Err(err) => match err {
            ECSError::EntityDoesntHaveComponent { .. } => world.get(entity, rotation()),
            ECSError::NoSuchEntity { .. } => Err(err),
        },
    }
}
