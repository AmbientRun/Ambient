use glam::{ivec2, IVec2, UVec2};
use kiwi_app::gpu;
use kiwi_core::asset_cache;
use kiwi_ecs::{EntityId, World};
use kiwi_gpu::texture::Texture;

use crate::{get_terrain_cell, terrain_state};

pub fn gather_terrain_cells(world: &World, encoder: &mut wgpu::CommandEncoder, terrain_map: &Texture, top_left_cell: IVec2, cells: UVec2) {
    for y in 0..cells.y as i32 {
        for x in 0..cells.x as i32 {
            if let Some(id) = get_terrain_cell(world, top_left_cell + ivec2(x, y)) {
                if let Ok(state) = world.get_ref(id, terrain_state()) {
                    encoder.copy_texture_to_texture(
                        wgpu::ImageCopyTexture {
                            texture: &state.heightmap.handle,
                            mip_level: 0,
                            origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
                            aspect: wgpu::TextureAspect::All,
                        },
                        wgpu::ImageCopyTexture {
                            texture: &terrain_map.handle,
                            mip_level: 0,
                            origin: wgpu::Origin3d {
                                x: (x * (state.size.texture_size() as i32 - 1)) as u32,
                                y: (y * (state.size.texture_size() as i32 - 1)) as u32,
                                z: 0,
                            },
                            aspect: wgpu::TextureAspect::All,
                        },
                        state.size.heightmap_extent(),
                    );
                }
            }
        }
    }
}

/// Spreads the whole heightmap onto each cells invididual heightmap
pub fn spread_terrain_cells(
    world: &mut World,
    encoder: &mut wgpu::CommandEncoder,
    heightmap: &Texture,
    normalmap: &Texture,
    top_left_cell: IVec2,
    cells: UVec2,
) -> Vec<EntityId> {
    let mut changed_cells = Vec::new();
    let _ = world.resource(gpu()).clone();
    for y in 0..cells.y as i32 {
        for x in 0..cells.x as i32 {
            if let Some(id) = get_terrain_cell(world, top_left_cell + ivec2(x, y)) {
                if let Ok(state) = world.get_ref(id, terrain_state()) {
                    encoder.copy_texture_to_texture(
                        wgpu::ImageCopyTexture {
                            texture: &heightmap.handle,
                            mip_level: 0,
                            origin: wgpu::Origin3d {
                                x: (x * (state.size.texture_size() as i32 - 1)) as u32,
                                y: (y * (state.size.texture_size() as i32 - 1)) as u32,
                                z: 0,
                            },
                            aspect: wgpu::TextureAspect::All,
                        },
                        wgpu::ImageCopyTexture {
                            texture: &state.heightmap.handle,
                            mip_level: 0,
                            origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
                            aspect: wgpu::TextureAspect::All,
                        },
                        state.size.heightmap_extent(),
                    );

                    // Note: we're not copying the border cells, because those normals will be wrong
                    let lx = if x == 0 { 1 } else { 0 };
                    let ly = if y == 0 { 1 } else { 0 };
                    let rx = if x >= cells.x as i32 - 1 { 1 } else { 0 };
                    let ry = if y >= cells.y as i32 - 1 { 1 } else { 0 };
                    encoder.copy_texture_to_texture(
                        wgpu::ImageCopyTexture {
                            texture: &normalmap.handle,
                            mip_level: 0,
                            origin: wgpu::Origin3d {
                                x: (x * (state.size.texture_size() as i32 - 1)) as u32 + lx,
                                y: (y * (state.size.texture_size() as i32 - 1)) as u32 + ly,
                                z: 0,
                            },
                            aspect: wgpu::TextureAspect::All,
                        },
                        wgpu::ImageCopyTexture {
                            texture: &state.normalmap.handle,
                            mip_level: 0,
                            origin: wgpu::Origin3d { x: lx, y: ly, z: 0 },
                            aspect: wgpu::TextureAspect::All,
                        },
                        wgpu::Extent3d {
                            width: state.size.texture_size() as u32 - lx - rx,
                            height: state.size.texture_size() as u32 - ly - ry,
                            depth_or_array_layers: 1,
                        },
                    );
                    normalmap.generate_mipmaps_with_encoder(world.resource(asset_cache()).clone(), encoder);
                    changed_cells.push(id);
                }
            }
        }
    }
    changed_cells
}
