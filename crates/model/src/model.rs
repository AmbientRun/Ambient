use std::{collections::HashMap, path::Path, sync::Arc};

use ambient_core::{
    asset_cache,
    bounding::{local_bounding_aabb, visibility_from, world_bounding_aabb, world_bounding_sphere},
    hierarchy::{children, parent},
    main_scene, name,
    transform::{
        fbx_complex_transform, fbx_post_rotation, fbx_pre_rotation, fbx_rotation_offset, fbx_rotation_pivot, fbx_scaling_offset,
        fbx_scaling_pivot, inv_local_to_world, local_to_parent, local_to_world, mesh_to_local, mesh_to_world, rotation, scale, translation,
    },
};
use ambient_ecs::{query, ComponentDesc, Entity, EntityId, World};
use ambient_renderer::{
    cast_shadows, color, gpu_primitives,
    lod::cpu_lod_visible,
    primitives,
    skinning::{self, Skin, SkinsBuffer, SkinsBufferKey},
};
use ambient_std::{
    asset_cache::{AssetCache, AsyncAssetKeyExt, SyncAssetKeyExt},
    asset_url::AbsAssetUrl,
    download_asset::AssetError,
    shapes::AABB,
};
use futures::future::join_all;
use glam::{Mat4, Vec3, Vec4};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use super::{
    animation_bind_id, animation_binder, is_model_node, model_animatable, model_loaded, model_skin_ix, model_skins,
    pbr_renderer_primitives_from_url,
};

pub enum ModelSpawnRoot {
    AttachTo(Vec<EntityId>),
    Spawn,
}
impl Default for ModelSpawnRoot {
    fn default() -> Self {
        ModelSpawnRoot::Spawn
    }
}
pub struct ModelSpawnOpts {
    pub root: ModelSpawnRoot,
    /// Spawn the model as a scene, and include an animation_binder
    pub animatable: Option<bool>,
    pub lod_group_states: bool,
    pub cast_shadows: bool,
    pub root_components: Entity,
}
impl Default for ModelSpawnOpts {
    fn default() -> Self {
        Self {
            root: ModelSpawnRoot::default(),
            animatable: None,
            lod_group_states: false,
            cast_shadows: true,
            root_components: Entity::new(),
        }
    }
}

/// This is a client side, spawnable, "3d model".
///
/// It's represented internally as a World, with some convetions which makes it
/// possible to load and spawn it again on the client side. Using this wrapper
/// instead of a raw World ensure those conventions are maintained.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model(pub World);
impl Model {
    #[cfg(not(target_os = "unknown"))]
    pub async fn from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        Ok(Self(World::from_file(path).await?))
    }
    pub fn from_slice(content: &[u8]) -> anyhow::Result<Self> {
        Ok(Self(World::from_slice(content)?))
    }
    pub async fn load(&mut self, assets: &AssetCache, model_url: &AbsAssetUrl) -> anyhow::Result<()> {
        for (id, prims) in query(pbr_renderer_primitives_from_url()).collect_cloned(&self.0, None) {
            self.0.remove_component(id, pbr_renderer_primitives_from_url()).unwrap();
            let prims = join_all(prims.into_iter().map(|prim| async move { prim.resolve(model_url)?.get(assets).await }).collect_vec())
                .await
                .into_iter()
                .collect::<Result<Vec<_>, AssetError>>()?
                .into_iter()
                .map(|x| (*x).clone())
                .collect_vec();
            self.0.add_component(id, primitives(), prims).unwrap();
        }
        Ok(())
    }
    pub fn name(&self) -> Option<&String> {
        self.0.resource_opt(name())
    }
    pub fn set_name(&mut self, name: &str) {
        self.0.add_resource(ambient_core::name(), name.to_string());
    }
    pub fn roots(&self) -> Vec<EntityId> {
        self.0.resource_opt(children()).cloned().unwrap_or_default()
    }
    pub fn animatable(&self) -> Option<bool> {
        self.0.resource_opt(model_animatable()).cloned()
    }
    pub fn aabb(&self) -> Option<AABB> {
        self.0.resource_opt(local_bounding_aabb()).cloned()
    }
    pub fn get_transform(&self) -> Option<Mat4> {
        self.0.resource_opt(local_to_parent()).cloned()
    }
    pub fn skins(&self) -> Option<&Vec<ModelSkin>> {
        self.0.resource_opt(model_skins())
    }

    pub fn spawn(&self, world: &mut World, opts: &ModelSpawnOpts) -> EntityId {
        self.batch_spawn(world, opts, 1).pop().unwrap()
    }
    pub fn batch_spawn(&self, world: &mut World, opts: &ModelSpawnOpts, count: usize) -> Vec<EntityId> {
        let mut root_components = opts.root_components.clone().with(main_scene(), ()).with_default(model_loaded());
        if let Some(aabb) = self.aabb() {
            root_components = root_components
                .with(local_bounding_aabb(), aabb)
                .with(world_bounding_aabb(), aabb)
                .with(world_bounding_sphere(), aabb.to_sphere())
        }

        // See README.md
        let animatable = opts.animatable.unwrap_or(self.animatable().unwrap_or(self.0.len() > 2));
        let spawn_as_scene = animatable || self.0.len() > 2;

        if spawn_as_scene {
            if animatable {
                root_components.set_self(animation_binder(), Default::default());
            }

            let skins_buffer_h = SkinsBufferKey.get(world.resource(asset_cache()));
            let mut skins_buffer = skins_buffer_h.lock();
            let mut entity_lookups = vec![HashMap::new(); count];

            let roots = match &opts.root {
                ModelSpawnRoot::AttachTo(entities) => {
                    for entity in entities.iter() {
                        let mut root_components = root_components.clone();
                        if !world.has_component(*entity, local_to_world()) {
                            root_components.set_self(local_to_world(), glam::Mat4::IDENTITY);
                        }
                        if !world.has_component(*entity, children()) {
                            root_components.set_self(children(), vec![]);
                        }
                        if world.has_component(*entity, name()) {
                            root_components.remove_self(name());
                        }

                        tracing::info!("Attaching to {:?} to {entity}", root_components.get(local_bounding_aabb()));

                        world.add_components(*entity, root_components).unwrap();
                    }
                    entities.clone()
                }
                ModelSpawnRoot::Spawn => {
                    if let Some(name_) = self.name() {
                        root_components.set_self(name(), name_.clone());
                    }
                    world.batch_spawn(root_components.with(children(), vec![]).with_default(local_to_world()), count)
                }
            };

            let transform_roots = if let Some(transform) = self.get_transform() {
                let transform_roots = world.batch_spawn(
                    Entity::new()
                        .with(parent(), EntityId::null())
                        .with(children(), vec![])
                        .with(local_to_parent(), transform)
                        .with_default(local_to_world())
                        .with_default(is_model_node()),
                    count,
                );
                for (transform, root) in transform_roots.iter().zip(roots.iter()) {
                    world.set(*transform, parent(), *root).unwrap();
                    world.set(*root, children(), vec![*transform]).unwrap();
                }
                transform_roots
            } else {
                roots.clone()
            };

            for node_id in &self.roots() {
                let childs = self.spawn_subtree(
                    world,
                    *node_id,
                    transform_roots.clone(),
                    roots.clone(),
                    &mut entity_lookups,
                    &mut skins_buffer,
                    opts,
                    count,
                );
                for (root, child) in transform_roots.iter().zip(childs.iter()) {
                    world.get_mut(*root, children()).unwrap().push(*child);
                }
            }

            for entity_lookup in &entity_lookups {
                for id in entity_lookup.values() {
                    if let Ok(children) = world.get_mut(*id, children()) {
                        for j in children.iter_mut() {
                            *j = *entity_lookup.get(j).unwrap();
                        }
                    }
                    if let Ok(joints) = world.get_mut(*id, skinning::joints()) {
                        for j in joints.iter_mut() {
                            *j = *entity_lookup.get(j).unwrap();
                        }
                    }
                }
            }

            if animatable {
                for (root, entity_lookups) in roots.iter().zip(entity_lookups.into_iter()) {
                    let anim_binds = entity_lookups
                        .into_iter()
                        .filter_map(|(id, entity)| (self.0.get_ref(id, animation_bind_id()).ok().map(|bid| (bid.clone(), entity))))
                        .collect();
                    world.add_component(*root, animation_binder(), anim_binds).unwrap();
                }
            }

            roots
        } else {
            let root_node = self.roots()[0];
            let root_ed = self.create_entity_data(root_node, opts, Some(self.get_transform().unwrap_or(Mat4::IDENTITY)));
            root_components.merge(root_ed);
            match &opts.root {
                ModelSpawnRoot::AttachTo(entities) => {
                    for entity in entities.iter() {
                        let mut root_components = root_components.clone();
                        if world.has_component(*entity, local_to_world()) {
                            root_components.remove_self(local_to_world());
                        } else {
                            root_components.set_self(local_to_world(), glam::Mat4::IDENTITY);
                        }
                        if world.has_component(*entity, name()) {
                            root_components.remove_self(name());
                        }
                        if world.has_component(*entity, translation()) {
                            root_components.remove_self(translation());
                        }
                        if world.has_component(*entity, scale()) {
                            root_components.remove_self(scale());
                        }
                        if world.has_component(*entity, rotation()) {
                            root_components.remove_self(rotation());
                        }
                        world.add_components(*entity, root_components).unwrap();
                    }
                    entities.clone()
                }
                ModelSpawnRoot::Spawn => {
                    if let Some(name_) = self.name() {
                        root_components.set_self(name(), name_.clone());
                    }
                    world.batch_spawn(root_components.with(local_to_world(), glam::Mat4::IDENTITY), count)
                }
            }
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn spawn_subtree(
        &self,
        world: &mut World,
        id: EntityId,
        parent_ids: Vec<EntityId>,
        root_ids: Vec<EntityId>,
        entity_lookups: &mut Vec<HashMap<EntityId, EntityId>>,
        skins_buffer: &mut SkinsBuffer,
        opts: &ModelSpawnOpts,
        count: usize,
    ) -> Vec<EntityId> {
        let mut ed = self
            .create_entity_data(id, opts, None)
            .with(parent(), EntityId::null())
            .with_default(is_model_node())
            .with(local_to_parent(), Mat4::IDENTITY);

        if let Some(skin_ix) = ed.remove_self(model_skin_ix()) {
            let skin = self.skins().unwrap()[skin_ix].clone();
            let count = skin.inverse_bind_matrices.len();
            ed.set_self(skinning::inverse_bind_matrices(), skin.inverse_bind_matrices.clone());
            ed.set_self(skinning::joints(), skin.joints);
            ed.set_self(skinning::joint_matrices(), vec![Mat4::IDENTITY; count]);
            ed.set_self(skinning::skin(), Skin::null());
            ed.set_self(inv_local_to_world(), Default::default());
        }

        if self.0.has_component(id, primitives()) {
            ed.set_self(visibility_from(), EntityId::null());
        }
        let entities = world.batch_spawn(ed, count);
        if let Ok(skin_ix) = self.0.get(id, model_skin_ix()) {
            let skin = self.skins().unwrap()[skin_ix].clone();
            for entity in &entities {
                let skin_buffer = skins_buffer.create(skin.inverse_bind_matrices.len() as u32);
                world.set(*entity, skinning::skin(), skin_buffer).unwrap();
            }
        }
        for (i, entity) in entities.iter().enumerate() {
            entity_lookups[i].insert(id, *entity);
        }

        for (&entity, &parent_id) in entities.iter().zip(parent_ids.iter()) {
            world.set(entity, parent(), parent_id).unwrap();
        }

        if self.0.has_component(id, primitives()) {
            for (&entity, &root_id) in entities.iter().zip(root_ids.iter()) {
                world.set(entity, visibility_from(), root_id).unwrap();
            }
        }
        if self.0.has_component(id, children()) {
            for c in self.0.get_ref(id, children()).unwrap().iter() {
                self.spawn_subtree(world, *c, entities.clone(), root_ids.clone(), entity_lookups, skins_buffer, opts, count);
            }
        }
        entities
    }

    pub fn transform(&mut self, transform: Mat4) {
        self.0.add_resource(local_to_parent(), transform * self.0.resource_opt(local_to_parent()).cloned().unwrap_or_default());
    }
    pub fn rotate_yup_to_zup(&mut self) {
        self.transform(Mat4::from_cols(Vec4::X, Vec4::Z, Vec4::Y, Vec4::W));
    }
    pub fn center(&mut self) {
        // TODO: Multiple-roots wont work with this
        for root in self.0.resource(children()).clone() {
            self.0.set(root, translation(), Vec3::ZERO).unwrap();
        }
    }
    pub fn get_entity_id_by_name(&self, node_name: &str) -> Option<EntityId> {
        query(name()).iter(&self.0, None).find_map(|x| if x.1 == node_name { Some(x.0) } else { None })
    }
    pub fn get_entity_id_by_bind_id(&self, bind_id: &str) -> Option<EntityId> {
        query(animation_bind_id()).iter(&self.0, None).find_map(|x| if x.1 == bind_id { Some(x.0) } else { None })
    }
    /// Remove matrices that doesn't need to be there for when the model is stored on disk
    pub fn remove_non_storage_matrices(&mut self) {
        for (id, _) in query(()).incl(local_to_world()).collect_cloned(&self.0, None) {
            if id != self.0.resource_entity() {
                self.0.remove_component(id, local_to_world()).unwrap();
            }
        }
        for (id, _) in query(()).incl(local_to_parent()).collect_cloned(&self.0, None) {
            if id != self.0.resource_entity() {
                self.0.remove_component(id, local_to_parent()).unwrap();
            }
        }
    }
    pub fn update_model_aabb(&mut self) {
        let model_transform = self.get_transform().unwrap_or_default();
        let aabbs = query(local_bounding_aabb())
            .iter(&self.0, None)
            .map(|(id, aabb)| {
                let mesh_to_model = model_transform
                    * self.0.get(id, local_to_world()).unwrap_or_default()
                    * self.0.get(id, mesh_to_local()).unwrap_or_default();
                aabb.transform(&mesh_to_model).to_aabb()
            })
            .collect_vec();
        self.0.add_resource(local_bounding_aabb(), AABB::unions(&aabbs).unwrap_or(AABB::ZERO));
    }

    /// Applies the base pose of this model to the loaded model in  the world
    pub fn apply_base_pose(&self, world: &mut World, id: EntityId) {
        if let Ok(bindings) = world.get_ref(id, animation_binder()).cloned() {
            for (node, bind_id) in query(animation_bind_id()).iter(&self.0, None) {
                if let Some(target) = bindings.get(bind_id) {
                    self.apply_transform_to_entity(node, world, *target, true);
                }
            }
        }
    }

    fn apply_transform_to_entity(&self, source_entity: EntityId, world: &mut World, id: EntityId, rotation_only: bool) {
        let mut ed = Entity::new();
        let mut remove = Vec::new();
        self.build_transform(source_entity, &mut ed, Some(&mut remove), rotation_only);
        world.remove_components(id, remove).ok();
        world.add_components(id, ed).ok();
    }

    fn build_transform(&self, node: EntityId, ed: &mut Entity, mut remove: Option<&mut Vec<ComponentDesc>>, rotation_only: bool) {
        if let Ok(rot) = self.0.get(node, rotation()) {
            ed.set_self(rotation(), rot);
        } else if let Some(remove) = &mut remove {
            remove.push(rotation().desc());
        }
        if !rotation_only {
            if let Ok(pos) = self.0.get(node, translation()) {
                ed.set_self(translation(), pos);
            } else if let Some(remove) = &mut remove {
                remove.push(translation().desc());
            }
            if let Ok(scl) = self.0.get(node, scale()) {
                ed.set_self(scale(), scl);
            } else if let Some(remove) = &mut remove {
                remove.push(scale().desc());
            }
        }

        // fbx
        if self.0.has_component(node, fbx_complex_transform()) {
            ed.set_self(fbx_complex_transform(), ());
        } else if let Some(remove) = &mut remove {
            remove.push(fbx_complex_transform().desc());
        }
        if let Ok(val) = self.0.get(node, fbx_rotation_offset()) {
            ed.set_self(fbx_rotation_offset(), val);
        } else if let Some(remove) = &mut remove {
            remove.push(fbx_rotation_offset().desc());
        }
        if let Ok(val) = self.0.get(node, fbx_rotation_pivot()) {
            ed.set_self(fbx_rotation_pivot(), val);
        } else if let Some(remove) = &mut remove {
            remove.push(fbx_rotation_pivot().desc());
        }
        if let Ok(val) = self.0.get(node, fbx_pre_rotation()) {
            ed.set_self(fbx_pre_rotation(), val);
        } else if let Some(remove) = &mut remove {
            remove.push(fbx_pre_rotation().desc());
        }
        if let Ok(val) = self.0.get(node, fbx_post_rotation()) {
            ed.set_self(fbx_post_rotation(), val);
        } else if let Some(remove) = &mut remove {
            remove.push(fbx_post_rotation().desc());
        }
        if !rotation_only {
            if let Ok(val) = self.0.get(node, fbx_scaling_offset()) {
                ed.set_self(fbx_scaling_offset(), val);
            } else if let Some(remove) = &mut remove {
                remove.push(fbx_scaling_offset().desc());
            }
            if let Ok(val) = self.0.get(node, fbx_scaling_pivot()) {
                ed.set_self(fbx_scaling_pivot(), val);
            } else if let Some(remove) = &mut remove {
                remove.push(fbx_scaling_pivot().desc());
            }
        }
    }
    fn create_entity_data(&self, node: EntityId, opts: &ModelSpawnOpts, single_mesh_transform: Option<Mat4>) -> Entity {
        let mut ed = self.0.clone_entity(node).unwrap().with_default(local_to_world());
        if let Some(mat) = single_mesh_transform {
            ed.set_self(
                mesh_to_local(),
                mat * self.0.get(node, local_to_world()).unwrap_or_default() * self.0.get(node, mesh_to_local()).unwrap_or_default(),
            );
            if let Some(bounding) = ed.get_mut(local_bounding_aabb()) {
                *bounding = bounding.transform(&mat).to_aabb();
            }
        } else {
            if let Ok(transform) = self.0.get(node, mesh_to_local()) {
                ed.set_self(mesh_to_local(), transform);
            }
            self.build_transform(node, &mut ed, None, false);
        }

        if self.0.has_component(node, primitives()) {
            ed.set_self(gpu_primitives(), Default::default());
            if !ed.contains(color()) {
                ed.set_self(color(), Vec4::ONE);
            }
            ed.set_self(main_scene(), ());
            ed.set_self(mesh_to_world(), Mat4::IDENTITY);
            if opts.lod_group_states {
                ed.set_self(cpu_lod_visible(), false);
            }
            if opts.cast_shadows {
                ed.set_self(cast_shadows(), ());
            }
        }
        ed
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSkin {
    pub inverse_bind_matrices: Arc<Vec<Mat4>>,
    pub joints: Vec<EntityId>,
}
