#[macro_use]
extern crate closure;
use std::{f32::consts::PI, num::NonZeroU32, sync::Arc, time::Instant};

use glam::{vec2, vec3, vec4, IVec2, Mat4, Quat, UVec2, Vec2, Vec3, Vec3Swizzles, Vec4};
use itertools::Itertools;
use kiwi_app::gpu;
use kiwi_core::{
    asset_cache,
    async_ecs::async_run,
    bounding::{local_bounding_aabb, world_bounding_aabb, world_bounding_sphere},
    camera::{get_active_camera, projection_view},
    main_scene, mesh, name, runtime, snap_to_ground,
    transform::{local_to_parent, local_to_world, mesh_to_world, rotation, scale, translation},
    FixedTimestepSystem,
};
use kiwi_ecs::{components, query, Commands, EntityData, EntityId, FnSystem, SystemGroup, World};
use kiwi_editor_derive::ElementEditor;
use kiwi_element::{element_tree, render_parented_with_component, Element, ElementComponent, ElementComponentExt, Group, Hooks};
use kiwi_gpu::{
    fill::FillerKey,
    gpu::GpuKey,
    mesh_buffer::GpuMesh,
    std_assets::PixelTextureKey,
    texture::{Texture, TextureReader},
    texture_loaders::TextureFromUrl,
};
use kiwi_meshes::{GridMesh, GridMeshKey};
use kiwi_physics::{
    collider::{collider_type, ColliderType},
    helpers::transform_entity_parts,
    main_physics_scene,
    physx::{character_controller, physics, physics_shape, rigid_static, Physics, PhysicsKey},
    PxActorUserData, PxShapeUserData,
};
use kiwi_renderer::{cast_shadows, color, gpu_primitives, lod::cpu_lod, material, primitives, renderer_shader, SharedMaterial};
use kiwi_std::{
    asset_cache::{AssetCache, SyncAssetKey, SyncAssetKeyExt},
    asset_url::AbsAssetUrl,
    friendly_id, log_result,
    shapes::{Sphere, AABB},
};
use ndarray::{s, Array3, ArrayView3, Axis};
use physxx::{
    AsPxActor, AsPxRigidActor, PxActor, PxActorFlag, PxHeightFieldDesc, PxHeightFieldGeometry, PxMaterial, PxPhysicsRef,
    PxQuantizedHeightFieldSamples, PxRigidActor, PxRigidStaticRef, PxShapeFlag, PxTransform, PxUserData,
};
use serde::{Deserialize, Serialize};
pub use terrain_shader::*;
use tracing::info_span;

use crate::terrain_shader::{TerrainMaterial, TerrainMaterialParams};

pub mod brushes;
mod gather_spread;
pub mod intents;
mod terrain_shader;
pub use gather_spread::*;
pub use intents::*;
use kiwi_network::ServerWorldExt;
use kiwi_ui::use_async_asset;

components!("terrain", {
    terrain: (),
    terrain_cell: (),

    terrain_lod_factor: f32,
    terrain_cell_diagonal: f32,
    terrain_lods: Vec<Arc<GpuMesh>>,

    terrain_cell_bounding: Sphere,

    terrain_material_def: TerrainMaterialDef,

    terrain_state_cpu: Arc<TerrainStateCpu>,
    terrain_state: TerrainState,
    terrain_world_cell: IVec2,
    terrain_should_send_to_server: Option<Instant>,
    terrain_cell_needs_cpu_download: bool,
    terrain_cell_version: i32,
});
pub fn init_all_components() {
    init_components();
    intents::init_components();
}

pub const TERRAIN_BASE: f32 = -30.;
pub(crate) static OLD_CONTENT_SERVER_URL: &str = "https://fra1.digitaloceanspaces.com/dims-content/";

pub fn get_terrain_cell(world: &World, cell: IVec2) -> Option<EntityId> {
    query((terrain_world_cell(),)).iter(world, None).find(|(_, (c,))| **c == cell).map(|(id, _)| id)
}

pub fn spawn_terrain(world: &mut World, terrain_compressed: Arc<TerrainStateCpu>, cell: IVec2) -> EntityId {
    let position = (cell.as_vec2() * TerrainSize::new().size_in_meters()).extend(TERRAIN_BASE);

    EntityData::new()
        .set(scale(), Vec3::ONE)
        .set(rotation(), Quat::IDENTITY)
        .set(translation(), position)
        .set_default(local_to_world())
        .set(collider_type(), ColliderType::Static)
        .set(terrain_state_cpu(), terrain_compressed)
        .set(name(), "Terrain".to_string())
        .set(terrain_world_cell(), cell)
        .set(terrain_cell_needs_cpu_download(), false)
        .spawn(world)
}

fn create_terrain_physics(world: &World, terrain_state: Arc<TerrainStateCpu>, position: Vec3, _cell: IVec2) -> PxRigidStaticRef {
    let scene = world.resource(main_physics_scene());
    let physics = world.resource(physics());
    let physics_material = PxTerrainMaterialKey.get(world.resource(asset_cache()));
    let actor = px_rigid_static_from_heightmap(physics, physics_material, &terrain_state.heightmap);
    actor.as_actor().set_user_data(PxActorUserData { serialize: false });
    actor.set_actor_flag(PxActorFlag::VISUALIZATION, false);
    actor.get_shapes()[0].set_flag(PxShapeFlag::VISUALIZATION, false);
    actor.get_shapes()[0].set_flag(PxShapeFlag::SCENE_QUERY_SHAPE, true);
    actor.set_global_pose(&PxTransform::from_translation(position), false);
    scene.add_actor(&actor);
    actor
}

pub fn terrain_gpu_to_cpu_system() -> SystemGroup {
    SystemGroup::new(
        "dims/terrain/terrain_gpu_to_cpu_system",
        vec![
            query((terrain_state_cpu(),)).excl(terrain_state()).to_system(|q, world, qs, _| {
                for (id, (state,)) in q.collect_cloned(world, qs) {
                    let state = state.to_gpu(world.resource(asset_cache()).clone());
                    world.add_component(id, terrain_state(), state.clone()).ok();
                }
            }),
            Box::new(FixedTimestepSystem::new(
                1.,
                Box::new(FnSystem::new(|world, _| {
                    let to_update = query((terrain_cell_needs_cpu_download(),))
                        .iter(world, None)
                        .filter_map(|(id, (update,))| if *update { Some(id) } else { None })
                        .collect_vec();

                    let async_run = world.resource(async_run()).clone();
                    let runtime = world.resource(runtime()).clone();
                    for id in to_update {
                        world.set(id, terrain_cell_needs_cpu_download(), false).ok();
                        if let Ok(terrain) = world.get_ref(id, terrain_state()) {
                            let terrain = terrain.reader();
                            let async_run = async_run.clone();
                            runtime.spawn(async move {
                                let terrain = Arc::new(terrain.read().await.unwrap());
                                async_run.run(move |world| {
                                    log_result!(world.set(id, terrain_state_cpu(), terrain));
                                    log_result!(world.add_component(id, terrain_should_send_to_server(), Some(Instant::now())));
                                    // async_commands.add_component(id, outline_recursive(), vec4(0.5, 0.5, 1., 1.));
                                });
                            });
                        }
                    }
                })),
            )),
        ],
    )
}

pub fn server_systems() -> SystemGroup {
    SystemGroup::new(
        "dims/terrain/server_systems",
        vec![
            query((terrain_state_cpu().changed(), translation(), terrain_world_cell())).to_system_with_name(
                "terrain",
                |q, world, qs, _| {
                    let updated = q.collect_cloned(world, qs);
                    let missing =
                        query((terrain_state_cpu(), translation(), terrain_world_cell())).excl(physics_shape()).collect_cloned(world, None);
                    let all =
                        updated.into_iter().chain(missing.into_iter()).sorted_by_key(|x| x.0).dedup_by(|x, y| x.0 == y.0).collect_vec();
                    for (id, (state, position, cell)) in all {
                        if let Ok(actor) = world.get_ref(id, rigid_static()) {
                            if let Some(scene) = actor.get_scene() {
                                scene.remove_actor(actor, false);
                                for shape in actor.get_shapes() {
                                    shape.remove_user_data::<PxShapeUserData>();
                                }
                                actor.as_actor().remove_user_data::<PxActorUserData>();
                                actor.release();
                            }
                        }
                        let body = create_terrain_physics(world, state.clone(), position, cell);
                        body.get_shapes()[0].set_user_data(PxShapeUserData { entity: id, ..Default::default() });

                        world
                            .add_components(
                                id,
                                EntityData::new()
                                    .set(physics_shape(), body.get_shapes()[0].clone())
                                    .set(rigid_static(), body)
                                    .set(collider_type(), ColliderType::Static),
                            )
                            .unwrap();
                    }
                },
            ),
            query((translation(), character_controller())).to_system_with_name("character", |q, world, qs, _| {
                for (_id, (pos, controller)) in q.iter(world, qs) {
                    if let Some(height) = get_terrain_height(world, pos.xy()) {
                        if pos.z < height - 0.5 {
                            controller.set_position(pos.xy().extend(height + 1.).as_dvec3());
                        }
                    }
                }
            }),
            query((terrain_state_cpu().changed(), translation())).to_system_with_name("raise_objects", |q, world, qs, _| {
                let mut new_transform = Vec::new();

                for (cell, (state, cell_pos)) in q.iter(world, qs) {
                    let _span = info_span!("raise_objects", ?cell, ?cell_pos).entered();
                    for (id, (pos, rot, scale, offset)) in query((translation(), rotation(), scale(), snap_to_ground())).iter(world, None) {
                        if state.contains(cell_pos.xy(), pos.xy()) {
                            // Move the entity or attachment position up to the new position
                            if let Some(height) = state.get_height(pos.xy() - cell_pos.xy()) {
                                let new_pos = vec3(pos.x, pos.y, height + cell_pos.z + offset);
                                tracing::info!(height, ?new_pos, "Moving entity {id} out of terrain");
                                new_transform.push((id, *pos, new_pos, *rot, *scale));
                            }
                        }
                    }
                }

                for (id, _, pos, rot, scale) in new_transform {
                    let _old_pos = world.get(id, translation()).unwrap();
                    log_result!(transform_entity_parts(world, id, pos, rot, scale));
                }
            }),
        ],
    )
}

pub fn client_systems() -> SystemGroup {
    SystemGroup::new(
        "dims/terrain/client_systems",
        vec![
            Box::new(intents::terrain_intent_client_system()),
            query((terrain_world_cell(),)).excl(terrain_state_cpu()).to_system(|q, world, qs, _| {
                for (id, _) in q.collect_cloned(world, qs) {
                    world.add_component(id, terrain_state_cpu(), Arc::new(TerrainStateCpu::empty())).ok();
                }
            }),
            query((terrain_world_cell(),)).excl(terrain_cell_version()).to_system(|q, world, qs, _| {
                for (id, _) in q.collect_cloned(world, qs) {
                    world.add_component(id, terrain_cell_version(), 0).ok();
                }
            }),
            Box::new(terrain_gpu_to_cpu_system()),
            query((terrain_state().changed(), translation())).to_system(|q, world, qs, _| {
                for (id, (state, pos)) in q.collect_cloned(world, qs) {
                    render_parented_with_component(
                        world,
                        id,
                        element_tree(),
                        Group(vec![Terrain { state: state.clone(), heightmap_position: pos.xy() }.el().set_default(local_to_parent())])
                            .el(),
                    );
                }
            }),
            query((terrain_lods(), cpu_lod(), local_to_world(), terrain_cell_bounding(), terrain_lod_factor(), terrain_cell_diagonal()))
                .incl(terrain_cell())
                .to_system(|q, world, qs, _| {
                    profiling::scope!("terrain.lod");
                    if let Some(main_camera) = get_active_camera(world, main_scene()) {
                        let pw = world.get(main_camera, projection_view()).unwrap_or(Mat4::IDENTITY);
                        let camera_position = pw.inverse().project_point3(vec3(0., 0., -1.));

                        let mut commands = Commands::new();
                        for (id, (lods, &current_lod, &local_to_world, bounding, &lod_factor, &cell_diagonal)) in q.iter(world, qs) {
                            let world_bounding = bounding.transform(&local_to_world);
                            let dist_from_camera = (world_bounding.center - camera_position).length() - world_bounding.radius;
                            let lod_base = (dist_from_camera.sqrt() * lod_factor).floor();
                            let lod_start = (lod_base / lod_factor).powf(2.);
                            let lod_dist = dist_from_camera - lod_start;
                            let min_lod = if lod_dist <= cell_diagonal { (lod_base as i32 - 1).max(0) as usize } else { lod_base as usize };

                            let new_lod = min_lod.min(lods.len() - 1);

                            if new_lod != current_lod {
                                commands.set(id, mesh(), lods[new_lod].clone());
                                commands.set(id, cpu_lod(), new_lod);
                            }
                        }
                        commands.apply(world).unwrap();
                    }
                }),
        ],
    )
}

pub fn ray_terrain_intersection(world: &World, origin: Vec3, dir: Vec3) -> Option<f32> {
    for (_, (rigid_static,)) in query((rigid_static(),)).incl(terrain_state_cpu()).iter(world, None) {
        let shapes = rigid_static.get_shapes();
        let geom = shapes[0].get_geometry();
        let hits = physxx::raycast(
            origin,
            dir,
            &geom,
            &shapes[0].get_global_pose(rigid_static.as_rigid_actor()),
            f32::MAX,
            physxx::PxHitFlags::POSITION,
            1,
        );
        if !hits.is_empty() {
            return Some(hits[0].distance);
        }
    }
    None
}

pub fn get_terrain_height(world: &World, pos: Vec2) -> Option<f32> {
    let origin = vec3(pos.x, pos.y, 10_000.);
    let dir = vec3(0., 0., -1.);
    ray_terrain_intersection(world, origin, dir).map(|dist| {
        let hitpoint = origin + dir * dist;
        hitpoint.z
    })
}

/// Find the terrain cell which contains the point
pub fn find_terrain_cell(world: &World, point: Vec2) -> Option<(EntityId, Vec3, &Arc<TerrainStateCpu>)> {
    query((translation(), terrain_state_cpu())).iter(world, None).find_map(|(id, (pos, state))| {
        if state.contains(pos.xy(), point) {
            Some((id, *pos, state))
        } else {
            None
        }
    })
}

pub fn get_terrain_height_blerp(world: &World, point: Vec2) -> Option<f32> {
    let (_, pos, state) = find_terrain_cell(world, point)?;
    let offset = point - pos.xy();
    let h = state.get_height(offset).unwrap();

    Some(h)
}

#[derive(Debug)]
pub struct PxTerrainMaterialKey;
impl SyncAssetKey<PxMaterial> for PxTerrainMaterialKey {
    fn load(&self, assets: AssetCache) -> PxMaterial {
        PxMaterial::new(PxPhysicsRef::get(), 0.5, 0.5, 0.6)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ElementEditor)]
pub struct TerrainSize {
    pub lods: usize,
    pub base_polygons: usize,
}
impl Default for TerrainSize {
    fn default() -> Self {
        Self { lods: 4, base_polygons: 8 }
    }
}
impl TerrainSize {
    pub fn new() -> Self {
        Self { ..Default::default() }
    }
    pub fn polygons_at_lod(&self, lod: usize) -> usize {
        let lod_max = self.lods - 1;
        self.base_polygons * 2usize.pow((lod_max - lod) as u32)
    }
    pub fn texture_size(&self) -> usize {
        self.polygons_at_lod(0) + 1
    }
    pub fn heightmap_extent(&self) -> wgpu::Extent3d {
        wgpu::Extent3d { width: self.texture_size() as u32, height: self.texture_size() as u32, depth_or_array_layers: TERRAIN_LAYERS }
    }
    pub fn normalmap_extent(&self) -> wgpu::Extent3d {
        wgpu::Extent3d { width: self.texture_size() as u32, height: self.texture_size() as u32, depth_or_array_layers: 1 }
    }
    pub fn polygon_size(&self) -> usize {
        self.polygons_at_lod(0)
    }
    pub fn size_in_meters(&self) -> f32 {
        self.polygon_size() as f32
    }
    pub fn cell_and_texel_from_position(&self, position: Vec2) -> (IVec2, UVec2) {
        let cell = (position / self.size_in_meters()).floor().as_ivec2();
        let offset = position - cell.as_vec2() * self.size_in_meters();
        let tc = offset / self.size_in_meters();
        (cell, (tc * (self.texture_size() as f32)).as_uvec2())
    }
}

pub const TERRAIN_LAYERS: u32 = 7;
#[repr(usize)]
pub enum TerrainLayers {
    Rock,
    Soil,
    Sediment,
    Hardness,
    HardnessStrataAmount,
    HardnessStrataWavelength,

    // Ignored for now
    Water,
    WaterOutflowL,
    WaterOutflowR,
    WaterOutflowT,
    WaterOutflowB,
    WaterVelocityX,
    WaterVelocityY,
}

pub fn wgsl_terrain_preprocess(source: impl Into<String>) -> String {
    wgsl_terrain_consts(source)
        .replace("#TERRAIN_FUNCS", &wgsl_terrain_consts(include_str!("terrain_funcs.wgsl")))
        .replace("#GET_HARDNESS", &wgsl_terrain_consts(include_str!("brushes/get_hardness.wgsl")))
}
fn wgsl_terrain_consts(source: impl Into<String>) -> String {
    let source: String = source.into();
    source
        .replace("#ROCK_LAYER", &(TerrainLayers::Rock as usize).to_string())
        .replace("#SOIL_LAYER", &(TerrainLayers::Soil as usize).to_string())
        .replace("#SEDIMENT_LAYER", &(TerrainLayers::Sediment as usize).to_string())
        .replace("#WATER_LAYER", &(TerrainLayers::Water as usize).to_string())
        .replace("#WATER_OUTFLOW_L_LAYER", &(TerrainLayers::WaterOutflowL as usize).to_string())
        .replace("#WATER_OUTFLOW_R_LAYER", &(TerrainLayers::WaterOutflowR as usize).to_string())
        .replace("#WATER_OUTFLOW_T_LAYER", &(TerrainLayers::WaterOutflowT as usize).to_string())
        .replace("#WATER_OUTFLOW_B_LAYER", &(TerrainLayers::WaterOutflowB as usize).to_string())
        .replace("#WATER_VELOCITY_X_LAYER", &(TerrainLayers::WaterVelocityX as usize).to_string())
        .replace("#WATER_VELOCITY_Y_LAYER", &(TerrainLayers::WaterVelocityY as usize).to_string())
        .replace("#HARDNESS_LAYER", &(TerrainLayers::Hardness as usize).to_string())
        .replace("#HARDNESS_STRATA_AMOUNT_LAYER", &(TerrainLayers::HardnessStrataAmount as usize).to_string())
        .replace("#HARDNESS_STRATA_WAVELENGTH_LAYER", &(TerrainLayers::HardnessStrataWavelength as usize).to_string())
        .replace("#TERRAIN_BASE", &TERRAIN_BASE.to_string())
}

#[derive(Debug, Clone)]
pub struct TerrainState {
    pub id: String,
    pub size: TerrainSize,
    pub heightmap: Arc<Texture>,
    pub normalmap: Arc<Texture>,
}
impl TerrainState {
    pub fn new_empty(assets: AssetCache, size: TerrainSize) -> Self {
        let gpu = GpuKey.get(&assets);

        let heightmap = Arc::new(Texture::new(
            gpu.clone(),
            &wgpu::TextureDescriptor {
                size: size.heightmap_extent(),
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::STORAGE_BINDING,
                label: Some("heightmap"),
            },
        ));

        let normalmap = Arc::new(Texture::new(
            gpu,
            &wgpu::TextureDescriptor {
                size: size.normalmap_extent(),
                mip_level_count: size.normalmap_extent().width.ilog2(),
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::STORAGE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: Some("normalmap"),
            },
        ));
        FillerKey { format: normalmap.format }.get(&assets).run(
            &normalmap.create_view(&wgpu::TextureViewDescriptor {
                base_mip_level: 0,
                mip_level_count: NonZeroU32::new(1),
                ..Default::default()
            }),
            normalmap.size,
            vec4(0., 0., 1., 0.),
        );
        normalmap.generate_mipmaps(assets);

        Self { id: friendly_id(), size, heightmap, normalmap }
    }
    pub async fn to_cpu(&self) -> Option<TerrainStateCpu> {
        self.reader().read().await
    }
    pub fn reader(&self) -> TerrainStateReader {
        TerrainStateReader { size: self.size.clone(), heightmap: self.heightmap.reader(), normalmap: self.normalmap.reader() }
    }
}

pub struct TerrainStateReader {
    size: TerrainSize,
    heightmap: TextureReader,
    normalmap: TextureReader,
}
impl TerrainStateReader {
    pub async fn read(&self) -> Option<TerrainStateCpu> {
        Some(TerrainStateCpu {
            size: self.size.clone(),
            heightmap: self.heightmap.read_array_f32().await?.remove_axis(Axis(3)),
            normalmap: self.normalmap.read_array_f32().await?.remove_axis(Axis(0)),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TerrainStateCpu {
    pub size: TerrainSize,
    pub heightmap: Array3<f32>,
    pub normalmap: Array3<f32>,
}
impl TerrainStateCpu {
    pub fn empty() -> Self {
        let size = TerrainSize::new();
        Self {
            heightmap: Array3::zeros((TERRAIN_LAYERS as usize, size.texture_size(), size.texture_size())),
            normalmap: Array3::zeros((size.texture_size(), size.texture_size(), 4)),
            size,
        }
    }

    pub fn shape(&self) -> &[usize] {
        self.heightmap.shape()
    }

    pub fn to_gpu(&self, assets: AssetCache) -> TerrainState {
        let state = TerrainState::new_empty(assets, self.size.clone());
        state.heightmap.write_array(&self.heightmap);
        state.normalmap.write_array(&self.normalmap);
        state
    }
    pub fn texel_from_world_offset(&self, offset: Vec2) -> IVec2 {
        let shape = self.heightmap.shape();
        let texel = offset * vec2(shape[1] as f32, shape[2] as f32) / self.size.size_in_meters();
        texel.as_ivec2()
    }
    pub fn get_slice_at_texel(&self, texel: IVec2) -> Option<ArrayView3<f32>> {
        let shape = self.heightmap.shape();
        if texel.x < 0 || texel.y < 0 || texel.x > shape[1] as i32 - 1 || texel.y > shape[2] as i32 - 1 {
            return None;
        }
        Some(self.heightmap.slice(s![..2, texel.y..(texel.y + 1), texel.x..(texel.x + 1)]))
    }
    pub fn get_slice_at_world_offset(&self, offset: Vec2) -> Option<ArrayView3<f32>> {
        self.get_slice_at_texel(self.texel_from_world_offset(offset))
    }
    pub fn get_height_at_texel(&self, texel: IVec2) -> Option<f32> {
        self.get_slice_at_texel(texel).map(|x| x.sum())
    }
    pub fn get_height_at_world_offset(&self, offset: Vec2) -> f32 {
        self.get_height_at_texel(self.texel_from_world_offset(offset)).unwrap_or_default()
    }

    pub fn contains(&self, pos: Vec2, point: Vec2) -> bool {
        let size = self.size.size_in_meters();
        point.x > pos.x && point.y > pos.y && point.x < pos.x + size && point.y < pos.y + size
    }

    /// Retrieves the *interpolated* value at the specified offset from the terrain cell.
    pub fn get_height(&self, offset: Vec2) -> Option<f32> {
        let shape = self.heightmap.shape();
        let texel = offset * vec2(shape[1] as f32, shape[2] as f32) / self.size.size_in_meters();
        let min = Vec2::ZERO;
        let max = vec2(shape[1] as f32 - 1.0, shape[2] as f32 - 1.0);

        if texel.x < min.x || texel.x > max.x || texel.y < min.y || texel.y > max.y {
            tracing::info!("Texel {texel} is out of bounds");
            return None;
        }

        let corners = [
            vec2(texel.x.floor(), texel.y.floor()), // sw
            vec2(texel.x.ceil(), texel.y.floor()),  // se
            vec2(texel.x.floor(), texel.y.ceil()),  // nw
            vec2(texel.x.ceil(), texel.y.ceil()),   //ne
        ];

        let t = texel.fract();

        let heights = [
            // Should never fail as the corners are clamped.
            self.get_height_at_texel(corners[0].as_ivec2()).unwrap(),
            self.get_height_at_texel(corners[1].as_ivec2()).unwrap(),
            self.get_height_at_texel(corners[2].as_ivec2()).unwrap(),
            self.get_height_at_texel(corners[3].as_ivec2()).unwrap(),
        ];

        let bot = heights[0] * (1.0 - t.x) + heights[1] * t.x;
        let top = heights[2] * (1.0 - t.x) + heights[3] * t.x;
        Some(bot * (1.0 - t.y) + top * t.y)
    }
}

// #[derive(Clone, Serialize, Deserialize, PartialEq)]
// pub struct TerrainStateCompressed {
//     pub size: TerrainSize,
//     pub heightmap: Vec<u8>,
//     pub heightmap_range: (f32, f32),
//     pub normalmap: Vec<Vec<u8>>,
// }
// impl From<&TerrainStateCpu> for TerrainStateCompressed {
//     fn from(state: &TerrainStateCpu) -> Self {
//         let heightmap_min = *state.heightmap.iter().min_by(|a, b| a.partial_cmp(&b).unwrap()).unwrap_or(&0.);
//         let heightmap_max = *state.heightmap.iter().max_by(|a, b| a.partial_cmp(&b).unwrap()).unwrap_or(&0.);
//         Self {
//             size: state.size.clone(),
//             heightmap: array2_to_png(state.heightmap.view(), heightmap_min, heightmap_max, 2),
//             heightmap_range: (heightmap_min, heightmap_max),
//             normalmap: array3_to_png_array(&state.normalmap, -1., 1., 1),
//             splatmap: array3_to_png_array(&state.splatmap, 0., 1., 1),
//         }
//     }
// }
// impl From<&TerrainStateCompressed> for TerrainStateCpu {
//     fn from(state: &TerrainStateCompressed) -> Self {
//         Self {
//             size: state.size.clone(),
//             heightmap: png_to_array2(&state.heightmap, state.heightmap_range.0, state.heightmap_range.1),
//             normalmap: png_array_to_array3(&state.normalmap, -1., 1.),
//             splatmap: png_array_to_array3(&state.splatmap, 0., 1.),
//         }
//     }
// }
// impl std::fmt::Debug for TerrainStateCompressed {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("TerrainStateCompressed")
//             .field("size", &self.size)
//             // .field("heightmap", &self.heightmap)
//             .field("heightmap_range", &self.heightmap_range)
//             // .field("normalmap", &self.normalmap)
//             // .field("splatmap", &self.splatmap)
//             .finish()
//     }
// }

#[derive(Debug, Clone)]
pub struct Terrain {
    pub state: TerrainState,
    pub heightmap_position: Vec2,
}
impl ElementComponent for Terrain {
    fn render(self: Box<Self>, world: &mut World, hooks: &mut Hooks) -> Element {
        let assets = world.resource(asset_cache()).clone();

        let _gpu = world.resource(gpu()).clone();

        let (material_def, set_material_def) =
            hooks.use_state_with(|| world.persisted_resource(terrain_material_def()).cloned().unwrap_or_default());

        let ground_textures = use_async_asset(hooks, world, TerrainTexturesKey { texs: material_def.build().textures })
            .and_then(|x| x.ok())
            .unwrap_or_else(|| PixelTextureKey::white().get(&assets));

        let noise_texture = use_async_asset(
            hooks,
            world,
            TextureFromUrl {
                url: AbsAssetUrl::parse(format!(
                    "{OLD_CONTENT_SERVER_URL}assets/models/{}",
                    "ArtStationSurfaces/VFX-HQ-Seamless-Noise-Pack-Vol1/Noise_002.png"
                ))
                .unwrap(),
                format: wgpu::TextureFormat::Rgba8Unorm,
            },
        )
        .and_then(|x| x.ok())
        .unwrap_or_else(|| PixelTextureKey::white().get(&assets));

        // let (ground_textures, set_ground_textures) =
        //     hooks.use_state_with(|| Arc::new(Texture::new_single_color_texture_array(gpu.clone(), vec![UVec4::ONE, UVec4::ONE])));
        // hooks.use_effect(
        //     world,
        //     material_def.clone(),
        //     closure!(clone material_def, |world| {
        //         let assets = world.resource(asset_cache()).clone();
        //         world.resource(runtime()).clone().spawn(async move {
        //             let build = material_def.build();
        //             let res = TerrainTexturesKey { texs: build.textures }.get(&assets).await;
        //             match res {
        //                 Ok(val) => set_ground_textures(val),
        //                 Err(err) => log::error!("Failed to load terrain textures: {:?}", err)
        //             }
        //         });
        //         Box::new(|_| {})
        //     }),
        // );
        hooks.use_frame(closure!(clone material_def, |world| {
            if let Some(def) = world.persisted_resource(terrain_material_def()) {
                if def != &material_def {
                    set_material_def(def.clone());
                }
            }
        }));

        let size_in_meters = self.state.size.size_in_meters();

        let lod_factor = 1. / 12.;
        let cell_diagonal = (size_in_meters * size_in_meters * 2.).sqrt();
        let heightmap_position = self.heightmap_position.extend(0.);
        let terrain_material =
            hooks.use_memo_with((self.state.id.clone(), ground_textures.id, material_def, noise_texture.id), |(_, _, material_def, _)| {
                SharedMaterial::new(TerrainMaterial::new(
                    assets.clone(),
                    TerrainMaterialParams { heightmap_position, lod_factor, cell_diagonal, _padding: Default::default() },
                    Arc::new(self.state.heightmap.create_view(&wgpu::TextureViewDescriptor::default())),
                    Arc::new(self.state.normalmap.create_view(&wgpu::TextureViewDescriptor::default())),
                    Arc::new(ground_textures.create_view(&wgpu::TextureViewDescriptor {
                        dimension: Some(wgpu::TextureViewDimension::D2Array),
                        ..Default::default()
                    })),
                    Arc::new(ground_textures.create_view(&wgpu::TextureViewDescriptor {
                        dimension: Some(wgpu::TextureViewDimension::D2Array),
                        ..Default::default()
                    })),
                    Arc::new(noise_texture.create_view(&wgpu::TextureViewDescriptor::default())),
                    material_def.clone(),
                ))
            });

        let lod_meshes = (0..self.state.size.lods)
            .map(|lod| {
                GridMeshKey(GridMesh {
                    n_vertices_height: self.state.size.polygons_at_lod(lod) + 1,
                    n_vertices_width: self.state.size.polygons_at_lod(lod) + 1,
                    size: glam::vec2(self.state.size.size_in_meters(), self.state.size.size_in_meters()),
                    ..GridMesh::default()
                })
                .get(&assets)
            })
            .collect_vec();

        let terrain_shader = TerrainShaderKey.get(&assets);

        let (height_min, height_max) = (0., 500.);
        let aabb = AABB { min: vec3(0., 0., height_min), max: vec3(size_in_meters, size_in_meters, height_max) };
        let bound_sphere = aabb.to_sphere();
        Element::new()
            .set(terrain(), ())
            .init_default(terrain_cell())
            .set(renderer_shader(), terrain_shader)
            .set(material(), terrain_material)
            .set(primitives(), vec![])
            .set_default(gpu_primitives())
            .set(main_scene(), ())
            .set(mesh(), lod_meshes[0].clone())
            .set(terrain_lods(), lod_meshes.clone())
            .set(terrain_lod_factor(), lod_factor)
            .set(terrain_cell_diagonal(), cell_diagonal)
            .set(cpu_lod(), 0_usize)
            .set(terrain_cell_bounding(), bound_sphere)
            .set(local_bounding_aabb(), aabb)
            .set(world_bounding_aabb(), aabb)
            .set(world_bounding_sphere(), bound_sphere)
            .set(color(), Vec4::ONE)
            .set_default(cast_shadows())
            .set_default(local_to_parent())
            .set_default(local_to_world())
            .init_default(mesh_to_world())
    }
}

pub fn px_rigid_static_from_heightmap(physics: &Physics, physics_material: PxMaterial, heightmap: &Array3<f32>) -> PxRigidStaticRef {
    let heightmap = heightmap.slice(s![0..2, .., ..]).sum_axis(Axis(0));
    let texture_size = heightmap.shape()[0];
    let total_size = texture_size - 1;

    let mut heightmap_samples = heightmap.reversed_axes();
    heightmap_samples.invert_axis(Axis(1));
    let mut quantized = PxQuantizedHeightFieldSamples::new_from_f32_array(heightmap_samples.as_standard_layout().as_slice().unwrap());
    for i in 0..quantized.samples.len() {
        quantized.samples[i].set_tesselation(true);
    }
    let hfd = PxHeightFieldDesc::new(texture_size as u32, texture_size as u32, &quantized.samples);

    let xy_scale = 1.;
    PxRigidStaticRef::new_with_geometry(
        physics.physics,
        &PxTransform::identity(),
        &PxHeightFieldGeometry::new(
            &mut physics.cooking.create_height_field(&physics.physics, &hfd),
            quantized.height_scale,
            xy_scale,
            xy_scale,
        ),
        &physics_material,
        &PxTransform::new(vec3(0., xy_scale * (total_size as f32), quantized.min_height), Quat::from_rotation_x(PI / 2.)),
    )
}
