// This module is partially complete and has not been wired up.
// We don't want dead code warnings from it in the meantime.
#![allow(dead_code)]

use std::sync::Arc;

use ambient_core::{asset_cache, gpu, mesh, transform::translation};
use ambient_ecs::{components, query, Entity, SystemGroup};
use ambient_gpu::{
    gpu::Gpu,
    shader_module::{BindGroupDesc, Shader, ShaderModule},
    typed_buffer::TypedBuffer,
};
use ambient_meshes::QuadMeshKey;
use ambient_renderer::{self, *};
use ambient_std::{asset_cache::*, cb, friendly_id, include_file};
use glam::*;
use noise::OpenSimplex;
use wgpu::{BindGroup, BufferUsages};

use self::tree::*;

mod tree;

pub use ambient_ecs::generated::components::core::rendering::sky;

// Temporary hack: expose the source of this shader so that it can be used by ambient_water when published
pub const ATMOSPHERIC_SCATTERING_SOURCE: &str = include_str!("atmospheric_scattering.wgsl");

components!("rendering", {
    cloud_state: CloudState,
});

#[derive(Debug, Clone)]
pub struct Clouds {}

#[derive(Clone)]
pub struct CloudState {
    tree: Octree,
}

const MAX_DEPTH: u32 = 20;
const DENSITY_THRESHOLD: f32 = 0.2;
const VOXEL_SIZE: f32 = 0.05;

impl CloudState {
    pub fn new(half_size: f32) -> Self {
        let generator = OpenSimplex::new(); // TODO enum noise
        let tree = OctreeInfo {
            max_depth: MAX_DEPTH,
            half_size,
            generator: Arc::new(generator),
            ..OctreeInfo::default()
        }
        .build();
        Self { tree }
    }
}

pub fn systems() -> SystemGroup {
    SystemGroup::new(
        "sky",
        vec![
            query(sky())
                .excl(renderer_shader())
                .to_system(|q, world, qs, _| {
                    let assets = world.resource(asset_cache()).clone();
                    let gpu = world.resource(gpu()).clone();
                    for (id, _) in q.collect_cloned(world, qs) {
                        let clouds = CloudState::new(100.0);

                        let material = CloudMaterial::new(&gpu, &assets, &clouds);

                        let data = Entity::new()
                            .with(
                                renderer_shader(),
                                cb(|assets, config| {
                                    CloudShaderKey {
                                        shadow_cascades: config.shadow_cascades,
                                    }
                                    .get(assets)
                                }),
                            )
                            .with(ambient_renderer::material(), SharedMaterial::new(material))
                            .with(cloud_state(), clouds)
                            .with(overlay(), ())
                            .with(mesh(), QuadMeshKey.get(&assets))
                            .with(primitives(), vec![])
                            .with_default(gpu_primitives_mesh())
                            .with_default(gpu_primitives_lod())
                            .with(translation(), vec3(0.0, 0.0, -1.0));
                        world.add_components(id, data).unwrap();
                    }
                }),
            // query_mut((cloud_state(),), (material(),)).with_commands(|q, w, qs, _, c| {
            //     let camera = get_active_camera(w, main_scene()).unwrap_or(EntityId::null());
            //     let cam_pos = w.get(camera, translation()).unwrap_or_default();
            //     let fov = w.get(camera, fovy()).unwrap_or_default();

            //     let assets = w.resource(asset_cache()).clone();
            //     for (e, (state,), (mat,)) in q.iter(w, qs) {
            //         // Update the tree LOD
            //         let (_, updates) = state.tree.update_topo(NodeIndex::root(), 0, VOXEL_SIZE, fov, cam_pos);

            //         // Write tree to gpu only if the tree changed
            //         if updates > 0 {
            //             let nodes = state.tree.nodes();

            //             let mat = mat.borrow_downcast::<CloudMaterial>();
            //             if mat.cloud_buffer.len() < nodes.len() as u64 {
            //                 c.set(e, material(), CloudMaterial::new(assets.clone(), state));
            //             } else {
            //                 mat.cloud_buffer.write(0, nodes);
            //             }
            //         }
            //     }
            // }),
            // query((cloud_state(),)).to_system(|q, w, qs, _| {
            //     // Visualize the clouds
            //     for (_, (state,)) in q.iter(w, qs) {
            //         if let Some(camera) = get_active_camera(w, main_scene()) {
            //             let mut ray = screen_ray(w, camera, Vec2::ZERO).unwrap();
            //             ray.dir *= -1.;
            //             let hit = state.tree.raycast(&ray, DENSITY_THRESHOLD);

            //             w.resource(gizmos()).scope(line_hash!()).draw(hit.into_iter().flatten()).draw(state.tree.gizmos(DENSITY_THRESHOLD));
            //         }
            //     }
            // }),
        ],
    )
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, PartialEq, Eq)]
struct CloudParams {
    count: i32,
}

#[derive(Debug)]
pub struct CloudMaterial {
    id: String,
    pub bind_group: wgpu::BindGroup,
    cloud_buffer: TypedBuffer<Node>,
}

impl CloudMaterial {
    pub fn new(gpu: &Gpu, assets: &AssetCache, state: &CloudState) -> Self {
        let cloud_buffer = TypedBuffer::new(
            gpu,
            "Cloud Buffer",
            state.tree.len().max(64) as usize,
            BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        );

        Self {
            id: friendly_id(),
            bind_group: gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &get_cloud_shader_layout().get(assets),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: cloud_buffer.buffer().as_entire_binding(),
                }],
                label: Some("CloudMaterial.bind_group"),
            }),
            cloud_buffer,
        }
    }
}

impl Material for CloudMaterial {
    fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    fn id(&self) -> &str {
        &self.id
    }
}

pub fn get_scatter_module() -> Arc<ShaderModule> {
    Arc::new(ShaderModule::new("Scatter", ATMOSPHERIC_SCATTERING_SOURCE))
}

fn get_cloud_shader_layout() -> BindGroupDesc<'static> {
    BindGroupDesc {
        entries: vec![wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: MATERIAL_BIND_GROUP.into(),
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CloudShaderKey {
    shadow_cascades: u32,
}

impl SyncAssetKey<Arc<RendererShader>> for CloudShaderKey {
    fn load(&self, assets: AssetCache) -> Arc<RendererShader> {
        let layout = get_cloud_shader_layout();

        let shader = include_file!("clouds.wgsl");

        let id = "cloud shader".to_string();
        Arc::new(RendererShader {
            shader: Shader::new(
                &assets,
                "clouds",
                &[GLOBALS_BIND_GROUP, MATERIAL_BIND_GROUP],
                &ShaderModule::new("clouds", shader)
                    .with_binding_desc(layout)
                    .with_dependencies(get_overlay_modules(&assets, self.shadow_cascades))
                    .with_dependency(get_scatter_module()),
            )
            .unwrap(),
            id,
            vs_main: "vs_main".to_string(),
            fs_forward_main: "fs_forward_main".to_string(),
            fs_shadow_main: "fs_shadow_main".to_string(),
            fs_outline_main: "fs_outlines_main".to_string(),
            transparent: true,
            double_sided: false,
            depth_write_enabled: true,
            transparency_group: 0,
        })
    }
}
