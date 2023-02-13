use std::sync::{atomic::AtomicI32, Arc};

use glam::{ivec2, IVec2, UVec2, Vec2, Vec3, Vec3Swizzles};
use kiwi_app::gpu;
use kiwi_core::{asset_cache, frame_index, map_seed};
use kiwi_ecs::{EntityId, World};
use kiwi_gpu::{gpu::GpuKey, std_assets::PixelTextureViewKey, texture::Texture};
use kiwi_network::ServerWorldExt;
use kiwi_std::asset_cache::{AssetCache, AsyncAssetKey, SyncAssetKeyExt};
use serde::{Deserialize, Serialize};

use super::{gather_terrain_cells, spread_terrain_cells, TerrainSize, TerrainStateCpu, TERRAIN_LAYERS};
use crate::{get_terrain_cell, spawn_terrain, terrain_cell_needs_cpu_download, terrain_cell_version, terrain_state};

mod flatten;
mod hydraulic_erosion;
mod init;
mod normalmap;
mod raise;
mod thermal_erosion;
mod water_sim;

use async_trait::async_trait;
pub use flatten::*;
pub use hydraulic_erosion::*;
pub use init::*;
pub use normalmap::*;
pub use raise::*;
pub use thermal_erosion::*;
pub use water_sim::*;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Brush {
    Raise,
    Lower,
    Flatten,
    Erode,
    Erode2,
    Thermal,
}
unsafe impl bytemuck::Pod for Brush {}
unsafe impl bytemuck::Zeroable for Brush {}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BrushSize(pub f32);
impl BrushSize {
    pub const TINY: Self = Self(1.);
    pub const SMALL: Self = Self(30.);
    pub const MEDIUM: Self = Self(100.);
    pub const LARGE: Self = Self(300.);

    fn radius(&self) -> f32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BrushStrength(pub f32);
impl BrushStrength {
    pub const SMALL: Self = Self(0.1);
    pub const MEDIUM: Self = Self(1.);
    pub const LARGE: Self = Self(10.);

    fn strength(&self) -> f32 {
        self.0
    }
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrushShape {
    Circle,
    Square,
}
unsafe impl bytemuck::Pod for BrushShape {}
unsafe impl bytemuck::Zeroable for BrushShape {}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BrushSmoothness(pub f32);

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BrushWGSL {
    pub center: Vec2,
    pub radius: f32,
    pub shape: BrushShape,
    pub smoothness: f32,
    pub amplitude: f32,
    pub _padding: UVec2,
}
impl Default for BrushWGSL {
    fn default() -> Self {
        Self { center: Vec2::ZERO, radius: 1., shape: BrushShape::Circle, smoothness: 1., amplitude: 0., _padding: Default::default() }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainBrushStroke {
    pub center: Vec2,
    pub layer: u32,
    pub brush: Brush,
    pub brush_size: BrushSize,
    pub brush_strength: BrushStrength,
    pub brush_shape: BrushShape,
    pub brush_smoothness: BrushSmoothness,
    pub start_position: Vec3,
    pub erosion: HydraulicErosionConfig,
}
impl TerrainBrushStroke {
    fn get_brush_cells(&self) -> (IVec2, IVec2) {
        let terrain = TerrainSize::new();
        let radius = self.brush_size.radius() * 1.2;
        let top_left_cell = ((self.center - radius) / terrain.size_in_meters()).floor().as_ivec2();
        let bottom_right_cell = ((self.center + radius) / terrain.size_in_meters()).ceil().as_ivec2();
        (top_left_cell, bottom_right_cell)
    }
    pub fn ensure_cells_exist(&self, world: &mut World) {
        let (top_left_cell, bottom_right_cell) = self.get_brush_cells();
        for y in top_left_cell.y..bottom_right_cell.y {
            for x in top_left_cell.x..bottom_right_cell.x {
                let cell = ivec2(x, y);
                if get_terrain_cell(world, cell).is_none() {
                    let terrain = TerrainStateCpu::empty();
                    spawn_terrain(world, Arc::new(terrain), cell);
                }
            }
        }
    }

    pub fn cells_exist(&self, world: &World) -> bool {
        let (top_left_cell, bottom_right_cell) = self.get_brush_cells();
        for y in top_left_cell.y..bottom_right_cell.y {
            for x in top_left_cell.x..bottom_right_cell.x {
                let cell = ivec2(x, y);
                if get_terrain_cell(world, cell).is_none() {
                    return false;
                }
            }
        }
        true
    }
    pub fn initial_island() -> Self {
        Self {
            center: Vec2::ZERO,
            layer: 0,
            brush: Brush::Raise,
            brush_size: BrushSize(300.0),
            brush_strength: BrushStrength(100.0),
            brush_shape: BrushShape::Circle,
            brush_smoothness: BrushSmoothness(1.),
            start_position: Default::default(),
            erosion: Default::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TerrainBrushKey;
#[async_trait]
impl AsyncAssetKey<Arc<TerrainBrush>> for TerrainBrushKey {
    async fn load(self, assets: AssetCache) -> Arc<TerrainBrush> {
        Arc::new(TerrainBrush::new(assets).await)
    }
}

#[derive(Clone, Debug)]
pub struct TerrainBrush {
    brush_raise_lower: Arc<RaiseBrush>,
    normals: Arc<NormalmapFromHeightmapCompute>,
    frame: Arc<AtomicI32>,
    intermediate_heightmap: Arc<Texture>,
    intermediate_normalmap: Arc<Texture>,
}

impl TerrainBrush {
    pub async fn new(assets: AssetCache) -> Self {
        let max_brush_size = 3000;
        let gpu = GpuKey.get(&assets);
        Self {
            brush_raise_lower: Arc::new(RaiseBrush::new(assets.clone()).await),
            normals: Arc::new(NormalmapFromHeightmapCompute::new(&gpu)),
            frame: Arc::new(AtomicI32::new(0)),
            intermediate_heightmap: Arc::new(Texture::new(
                gpu.clone(),
                &wgpu::TextureDescriptor {
                    label: Some("Terrain brush heightmap"),
                    size: wgpu::Extent3d { width: max_brush_size, height: max_brush_size, depth_or_array_layers: TERRAIN_LAYERS },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::R32Float,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::COPY_SRC
                        | wgpu::TextureUsages::STORAGE_BINDING,
                },
            )),
            intermediate_normalmap: Arc::new(Texture::new(
                gpu.clone(),
                &wgpu::TextureDescriptor {
                    label: Some("Terrain brush normalmap"),
                    size: wgpu::Extent3d { width: max_brush_size, height: max_brush_size, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba32Float,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::COPY_SRC
                        | wgpu::TextureUsages::STORAGE_BINDING,
                },
            )),
        }
    }
    #[profiling::function]
    pub fn apply(&self, world: &mut World, stroke: TerrainBrushStroke) -> Vec<EntityId> {
        let map_globals = world.persisted_resource_entity().unwrap();
        let seed = world.get(map_globals, map_seed()).unwrap();

        let (top_left_cell, bottom_right_cell) = stroke.get_brush_cells();
        let TerrainBrushStroke { center, layer, brush, brush_size, brush_strength, brush_smoothness, brush_shape, start_position, erosion } =
            stroke;
        let gpu = world.resource(gpu()).clone();
        let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let terrain = TerrainSize::new();
        let mut cells_size = bottom_right_cell - top_left_cell;
        cells_size.x = cells_size.x.max(1);
        cells_size.y = cells_size.y.max(1);
        let cells_size = cells_size.as_uvec2();
        let texture_size = cells_size * (terrain.texture_size() - 1) as u32 + 1;
        let heightmap_world_size = cells_size.as_vec2() * terrain.size_in_meters();
        let heightmap_world_texel_size = heightmap_world_size.x / (texture_size.x as f32 - 1.);
        gather_terrain_cells(world, &mut encoder, &self.intermediate_heightmap, top_left_cell, cells_size);
        match brush {
            Brush::Raise | Brush::Lower => {
                let amount = brush_strength.strength();
                let params = RaiseBrushParams {
                    heightmap_world_position: top_left_cell.as_vec2() * terrain.size_in_meters(),
                    heightmap_world_texel_size,
                    brush: BrushWGSL {
                        center,
                        radius: brush_size.radius(),
                        shape: brush_shape,
                        amplitude: if brush == Brush::Raise { amount } else { -amount },
                        smoothness: brush_smoothness.0,
                        _padding: Default::default(),
                    },
                    layer: layer as i32,
                    ..Default::default()
                };
                self.brush_raise_lower.run(
                    &gpu,
                    &mut encoder,
                    &self.intermediate_heightmap.create_view(&Default::default()),
                    texture_size,
                    &RaiseBrushConfig { params, seed },
                );
            }
            Brush::Flatten => {
                let brush = FlattenBrush::new(&gpu);
                let mut start_heightmap = PixelTextureViewKey::white().get(world.resource(asset_cache()));
                let (start_cell, start_texel) = TerrainSize::new().cell_and_texel_from_position(start_position.xy());
                if let Some(id) = get_terrain_cell(world, start_cell) {
                    if let Ok(state) = world.get_ref(id, terrain_state()) {
                        start_heightmap = Arc::new(state.heightmap.create_view(&Default::default()));
                    }
                }
                let params = FlattenBrushParams {
                    heightmap_world_position: top_left_cell.as_vec2() * terrain.size_in_meters(),
                    heightmap_world_texel_size,
                    brush: BrushWGSL {
                        center,
                        radius: brush_size.radius(),
                        shape: brush_shape,
                        amplitude: brush_strength.strength(),
                        smoothness: brush_smoothness.0,
                        _padding: Default::default(),
                    },
                    start_texel,
                    _padding: Default::default(),
                };
                brush.run(
                    &gpu,
                    &mut encoder,
                    &self.intermediate_heightmap.create_view(&Default::default()),
                    &start_heightmap,
                    texture_size,
                    &params,
                );
            }
            Brush::Erode => {
                let mut config = erosion;
                // config.drops_per_m2 = match brush_strength {
                //     BrushStrength::Small => 0.01,
                //     BrushStrength::Medium => 0.1,
                //     BrushStrength::Large => 1.4,
                // };
                config.params.heightmap_size = texture_size.as_ivec2();
                config.brush_radius = brush_size.radius();
                config.brush_position = center - top_left_cell.as_vec2() * terrain.size_in_meters();
                let brush = HydraulicErosionCompute::new(&gpu);
                brush.run(&gpu, &mut encoder, &self.intermediate_heightmap.create_view(&Default::default()), texture_size, &config);

                // for i in 0..4 {
                //     let brush = ThermalErosionCompute::new(&gpu);
                //     let mut config = ThermalErosionConfig::default();
                //     config.params = ThermalErosionParams {
                //         heightmap_world_position: top_left_cell.as_f32() * terrain.size_in_meters(),
                //         heightmap_world_size: cells_size.as_f32() * terrain.size_in_meters(),
                //         heightmap_texture_size: texture_size.as_i32(),
                //         brush_position: center,
                //         brush_radius: brush_size.radius(),
                //         frame: i,
                //         _padding: Default::default(),
                //     };
                //     brush.run(&gpu, &mut encoder, &self.intermediate_heightmap.create_view(&Default::default()), texture_size, &config);
                // }
            }
            Brush::Erode2 => {
                let brush = WaterSimCompute::new(&gpu);
                let mut config = WaterSimConfig::default();
                config.params.frame = *world.resource(frame_index()) as i32;
                brush.run(&gpu, &mut encoder, &self.intermediate_heightmap.create_view(&Default::default()), texture_size, &config);
            }
            Brush::Thermal => {
                let brush = ThermalErosionCompute::new(&gpu);
                let config = ThermalErosionConfig {
                    params: ThermalErosionParams {
                        heightmap_world_position: top_left_cell.as_vec2() * terrain.size_in_meters(),
                        heightmap_world_size: cells_size.as_vec2() * terrain.size_in_meters(),
                        heightmap_texture_size: texture_size.as_ivec2(),
                        brush_position: center,
                        brush_radius: brush_size.radius(),
                        frame: self.frame.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
                        _padding: Default::default(),
                    },
                };
                brush.run(&gpu, &mut encoder, &self.intermediate_heightmap.create_view(&Default::default()), texture_size, &config);
            }
        }
        self.normals.run(
            &gpu,
            &mut encoder,
            &self.intermediate_heightmap.create_view(&Default::default()),
            &self.intermediate_normalmap.create_view(&Default::default()),
            texture_size,
        );
        let changed_cells = spread_terrain_cells(
            world,
            &mut encoder,
            &self.intermediate_heightmap,
            &self.intermediate_normalmap,
            top_left_cell,
            cells_size,
        );

        gpu.queue.submit(Some(encoder.finish()));
        for id in &changed_cells {
            world.add_component(*id, terrain_cell_needs_cpu_download(), true).ok();
            *world.get_mut(*id, terrain_cell_version()).unwrap() += 1;
        }

        changed_cells
    }
}
