use std::{f32::INFINITY, sync::Arc};

use ambient_core::{
    bounding::world_bounding_sphere,
    camera::{shadow_cameras_from_world, Camera},
    gpu_components,
    gpu_ecs::{GpuComponentFormat, GpuWorldUpdater},
    player::local_user_id,
};
use ambient_ecs::{ArchetypeFilter, World};
use ambient_gpu::{
    gpu::Gpu,
    shader_module::{BindGroupDesc, ShaderIdent, ShaderModule},
    typed_buffer::TypedBuffer,
};
use ambient_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    include_file,
    shapes::Plane,
};
use glam::{Mat4, UVec3, Vec2, Vec3, Vec3Swizzles, Vec4};
use wgpu::{BindGroupLayout, BindGroupLayoutEntry, BindingType, BufferBindingType, ShaderStages};

use crate::{get_sun_light_direction, RendererConfig};

gpu_components! {
    world_bounding_sphere() => renderer_cameras_visible: GpuComponentFormat::Mat4,
}

const CULLING_BIND_GROUP: &str = "LODDING_BIND_GROUP";

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct CullCamera {
    pub view: Mat4,
    pub position: Vec4,
    pub frustum_right: Plane,
    pub frustum_top: Plane,
    pub orthographic_size: Vec2,
    pub frustum_near: f32,
    pub frustum_far: f32,
    pub cot_fov_2: f32,
    pub _padding: UVec3,
}

impl From<Camera> for CullCamera {
    fn from(camera: Camera) -> Self {
        let frustum = camera.projection.view_space_frustum();

        Self {
            view: camera.view,
            position: camera.position().extend(1.),
            frustum_right: frustum.right,
            frustum_top: frustum.top,
            orthographic_size: camera
                .projection
                .orthographic_size()
                .unwrap_or(Vec3::ZERO)
                .xy(),
            frustum_near: camera.projection.near(),
            frustum_far: camera.projection.far().unwrap_or(INFINITY),
            cot_fov_2: 1. / (camera.projection.fovy().unwrap_or(1.) / 2.).tan(),
            _padding: Default::default(),
        }
    }
}

pub const MAX_SHADOW_CASCADES: u32 = 6;

#[repr(C)]
#[derive(Debug, Clone, Default, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CullingParams {
    pub main_camera: CullCamera,
    pub shadow_cameras: [CullCamera; MAX_SHADOW_CASCADES as usize],
    pub lod_cutoff_scaling: f32,
    pub _padding: UVec3,
}

pub struct Culling {
    config: RendererConfig,
    updater: GpuWorldUpdater,
    params: TypedBuffer<CullingParams>,
    layout: Arc<BindGroupLayout>,
}

fn get_culling_layout() -> BindGroupDesc<'static> {
    BindGroupDesc {
        label: CULLING_BIND_GROUP.into(),
        entries: vec![BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    }
}

impl Culling {
    pub fn new(gpu: &Gpu, assets: &AssetCache, config: RendererConfig) -> Self {
        log::debug!("Setting up culling");
        let module = ShaderModule::new("CullingParams", include_file!("culling.wgsl"))
            .with_ident(ShaderIdent::constant(
                "SHADOW_CASCADES",
                config.shadow_cascades,
            ))
            .with_ident(ShaderIdent::constant(
                "MAX_SHADOW_CASCADES",
                MAX_SHADOW_CASCADES,
            ))
            .with_binding_desc(get_culling_layout());

        Self {
            updater: GpuWorldUpdater::new(
                gpu,
                assets.clone(),
                "Culling".to_string(),
                ArchetypeFilter::new()
                    .incl(world_bounding_sphere())
                    .incl(config.scene),
                vec![Arc::new(module)],
                &[CULLING_BIND_GROUP],
                "update(entity_loc);",
            ),
            params: TypedBuffer::new(
                gpu,
                "Culling.params",
                1,
                wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::UNIFORM,
            ),
            config,
            layout: get_culling_layout().get(assets),
        }
    }

    #[ambient_profiling::function]
    pub fn run<'a>(&mut self, gpu: &Gpu, encoder: &'a mut wgpu::CommandEncoder, world: &World) {
        let main_camera = if let Some(camera) = Camera::get_active(
            world,
            self.config.scene,
            world.resource_opt(local_user_id()),
        ) {
            camera
        } else {
            // log::warn!("No valid camera");
            return;
        };

        let mut params = CullingParams {
            lod_cutoff_scaling: self.config.lod_cutoff_scaling,
            main_camera: main_camera.into(),
            ..Default::default()
        };
        if self.config.shadow_cascades > 0 {
            let shadow_cameras = shadow_cameras_from_world(
                world,
                self.config.shadow_cascades,
                self.config.shadow_map_resolution,
                get_sun_light_direction(world, self.config.scene),
                self.config.scene,
                world.resource_opt(local_user_id()),
            );
            #[allow(clippy::needless_range_loop)]
            for i in 0..(self.config.shadow_cascades as usize) {
                params.shadow_cameras[i] = shadow_cameras[i].clone().into();
            }
        }

        self.params.fill(gpu, &[params], |_| {});

        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.params.buffer().as_entire_binding(),
            }],
        });

        self.updater
            .run_with_encoder(gpu, encoder, world, &[&bind_group]);
    }
}
