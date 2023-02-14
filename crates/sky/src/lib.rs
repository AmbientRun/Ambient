use std::sync::Arc;

use glam::*;
use kiwi_core::{
    asset_cache,
    camera::{fovy, get_active_camera, screen_ray},
    main_scene, mesh,
    transform::translation,
};
use kiwi_ecs::{components, query, query_mut, EntityId, SystemGroup};
use kiwi_element::{Element, ElementComponent, Hooks};
use kiwi_gizmos::gizmos;
use kiwi_gpu::{
    gpu::GpuKey,
    shader_module::{BindGroupDesc, Shader, ShaderModule},
    typed_buffer::TypedBuffer,
};
use kiwi_meshes::QuadMeshKey;
use kiwi_renderer::{self, *};
use kiwi_std::{asset_cache::*, cb, friendly_id, include_file, line_hash};
use noise::OpenSimplex;
use wgpu::{BindGroup, BufferUsages};

use self::tree::*;

mod tree;

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
        let tree = OctreeInfo { max_depth: MAX_DEPTH, half_size, generator: Arc::new(generator), ..OctreeInfo::default() }.build();
        Self { tree }
    }
}

pub fn clouds_system() -> SystemGroup {
    SystemGroup::new(
        "Cloud system",
        vec![
            query_mut((cloud_state(),), (material(),)).with_commands(|q, w, qs, _, c| {
                let camera = get_active_camera(w, main_scene()).unwrap_or(EntityId::null());
                let cam_pos = w.get(camera, translation()).unwrap_or_default();
                let fov = w.get(camera, fovy()).unwrap_or_default();

                let assets = w.resource(asset_cache()).clone();
                for (e, (state,), (mat,)) in q.iter(w, qs) {
                    // Update the tree LOD
                    let (_, updates) = state.tree.update_topo(NodeIndex::root(), 0, VOXEL_SIZE, fov, cam_pos);

                    // Write tree to gpu only if the tree changed
                    if updates > 0 {
                        let nodes = state.tree.nodes();

                        let mat = mat.borrow_downcast::<CloudMaterial>();
                        if mat.cloud_buffer.len() < nodes.len() as u64 {
                            c.set(e, material(), CloudMaterial::new(assets.clone(), state));
                        } else {
                            mat.cloud_buffer.write(0, nodes);
                        }
                    }
                }
            }),
            query((cloud_state(),)).to_system(|q, w, qs, _| {
                // Visualize the clouds
                for (_, (state,)) in q.iter(w, qs) {
                    if let Some(camera) = get_active_camera(w, main_scene()) {
                        let mut ray = screen_ray(w, camera, Vec2::ZERO).unwrap();
                        ray.dir *= -1.;
                        let hit = state.tree.raycast(&ray, DENSITY_THRESHOLD);

                        w.resource(gizmos()).scope(line_hash!()).draw(hit.into_iter().flatten()).draw(state.tree.gizmos(DENSITY_THRESHOLD));
                    }
                }
            }),
        ],
    )
}

impl ElementComponent for Clouds {
    fn render(self: Box<Self>, world: &mut kiwi_ecs::World, _: &mut Hooks) -> Element {
        let assets = world.resource(asset_cache()).clone();

        let clouds = CloudState::new(100.0);

        let material = CloudMaterial::new(assets.clone(), &clouds);

        Element::new()
            .init(renderer_shader(), cb(|assets, config| CloudShaderKey { shadow_cascades: config.shadow_cascades }.get(&assets)))
            .init(kiwi_renderer::material(), SharedMaterial::new(material))
            .init(cloud_state(), clouds)
            .init(overlay(), ())
            .init(mesh(), QuadMeshKey.get(&assets))
            .init(primitives(), vec![])
            .init_default(gpu_primitives())
            .init(translation(), vec3(0.0, 0.0, -1.0))
    }
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
    pub fn new(assets: AssetCache, state: &CloudState) -> Self {
        let gpu = GpuKey.get(&assets);
        let shader = CloudShaderKey { shadow_cascades: 1 }.get(&assets);

        let cloud_buffer = TypedBuffer::new(
            gpu.clone(),
            "Cloud Buffer",
            state.tree.len().max(64) as u64,
            0,
            BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        );

        Self {
            id: friendly_id(),
            bind_group: gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: shader.material_layout(),
                entries: &[wgpu::BindGroupEntry { binding: 0, resource: cloud_buffer.buffer().as_entire_binding() }],
                label: Some("CloudMaterial.bind_group"),
            }),
            cloud_buffer,
        }
    }
}

impl Material for CloudMaterial {
    fn bind(&self) -> &BindGroup {
        &self.bind_group
    }

    fn id(&self) -> &str {
        &self.id
    }
}

pub fn get_scatter_module() -> ShaderModule {
    ShaderModule::from_str("Scatter", include_file!("atmospheric_scattering.wgsl"))
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CloudShaderKey {
    shadow_cascades: u32,
}

impl SyncAssetKey<Arc<RendererShader>> for CloudShaderKey {
    fn load(&self, assets: AssetCache) -> Arc<RendererShader> {
        let layout = BindGroupDesc {
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
        };

        let shader = include_file!("clouds.wgsl");

        let id = "cloud shader".to_string();
        Arc::new(RendererShader {
            shader: Shader::from_modules(
                &assets,
                id.clone(),
                [
                    &get_overlay_module(&assets, self.shadow_cascades),
                    &get_scatter_module(),
                    &ShaderModule::new("Clouds", shader, vec![layout.into()]),
                ],
            ),
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
