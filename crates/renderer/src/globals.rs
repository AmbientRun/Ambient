use std::{sync::Arc, time::Instant};

use ambient_core::{
    camera::{far, fog, get_active_camera, projection_view},
    transform::{get_world_position, get_world_rotation, local_to_world},
};
use ambient_ecs::{Component, ECSError, World};
use ambient_gpu::{
    gpu::{Gpu, GpuKey},
    shader_module::BindGroupDesc,
    std_assets::DefaultSamplerKey,
    texture::{Texture, TextureView},
};
use ambient_std::asset_cache::{AssetCache, SyncAssetKeyExt};
use glam::{vec3, Mat4, UVec2, Vec3, Vec4};
use wgpu::BindGroup;

use super::{fog_color, get_active_sun, light_ambient, light_diffuse, RenderTarget, ShadowCameraData};
use crate::{fog_density, fog_height_falloff};

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[allow(dead_code)]
pub struct ShaderDebugParams {
    pub metallic_roughness: f32,
    pub normals: f32,
    pub shading: f32,
    padding: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GlobalParams {
    pub projection_view: Mat4,
    pub inv_projection_view: Mat4,
    pub camera_position: Vec4,
    pub camera_forward: Vec3,
    pub camera_far: f32,
    pub sun_direction: Vec4,
    pub sun_diffuse: Vec4,
    pub sun_ambient: Vec4,
    pub fog_color: Vec4,
    pub forward_camera_position: Vec4, // This is relevant when rendering shadow maps, in which the camera_position is the shadow cameras position
    pub fog: i32,
    pub time: f32,
    pub fog_height_falloff: f32,
    pub fog_density: f32,
    pub debug_params: ShaderDebugParams,
}

impl Default for GlobalParams {
    fn default() -> Self {
        Self {
            projection_view: Default::default(),
            inv_projection_view: Default::default(),
            camera_position: Vec4::new(1.0, 0.0, 0.2, 0.0),
            camera_forward: Vec3::X,
            camera_far: 1e6,
            sun_direction: default_sun_direction().extend(0.),
            sun_diffuse: Vec4::ONE * 3.0,
            sun_ambient: Vec4::ONE * 0.05,
            fog_color: Vec4::ONE * 0.2,
            forward_camera_position: Vec4::new(1.0, 0.0, 0.2, 0.0),
            fog: 0,
            time: 0.,
            fog_height_falloff: 0.5,
            fog_density: 0.5,
            debug_params: Default::default(),
        }
    }
}

pub fn default_sun_direction() -> Vec3 {
    vec3(-0.2, 1., 1.).normalize()
}

pub fn globals_layout() -> BindGroupDesc {
    BindGroupDesc {
        entries: vec![
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    sample_type: wgpu::TextureSampleType::Depth,
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 6,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Depth,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 7,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
        ],
        label: "GLOBALS_BIND_GROUP".into(),
    }
}

pub(crate) struct ForwardGlobals {
    gpu: Arc<Gpu>,
    buffer: wgpu::Buffer,
    shadow_cameras_buffer: wgpu::Buffer,
    shadow_sampler: wgpu::Sampler,
    dummy_shadow_texture: TextureView,
    pub(crate) params: GlobalParams,
    scene: Component<()>,
    start_time: Instant,
    layout: Arc<wgpu::BindGroupLayout>,
}

impl ForwardGlobals {
    pub fn new(gpu: Arc<Gpu>, layout: Arc<wgpu::BindGroupLayout>, shadow_cascades: u32, scene: Component<()>) -> Self {
        let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ForwardGlobals.buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: std::mem::size_of::<GlobalParams>() as u64,
            mapped_at_creation: false,
        });
        let shadow_cameras_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ForwardGlobals.shadow_cameras_buffer"),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            size: shadow_cascades as u64 * std::mem::size_of::<ShadowCameraData>() as u64,
            mapped_at_creation: false,
        });

        let shadow_sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::GreaterEqual),
            ..Default::default()
        });

        let params = GlobalParams::default();

        Self {
            buffer,
            shadow_cameras_buffer,
            shadow_sampler,
            dummy_shadow_texture: create_dummy_shadow_texture(gpu.clone()).create_view(&Default::default()),
            params,
            gpu,
            scene,
            start_time: Instant::now(),
            layout,
        }
    }

    pub fn create_bind_group(&self, assets: AssetCache, shadow_texture: Option<&TextureView>, solids_frame: &RenderTarget) -> BindGroup {
        self.gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::Sampler(&DefaultSamplerKey.get(&assets)) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Buffer(self.buffer.as_entire_buffer_binding()) },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(self.shadow_cameras_buffer.as_entire_buffer_binding()),
                },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&self.shadow_sampler) },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(shadow_texture.unwrap_or(&self.dummy_shadow_texture)),
                },
                wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::TextureView(&solids_frame.color_buffer_view) },
                wgpu::BindGroupEntry { binding: 6, resource: wgpu::BindingResource::TextureView(&solids_frame.depth_buffer_view) },
                wgpu::BindGroupEntry { binding: 7, resource: wgpu::BindingResource::TextureView(&solids_frame.normals_quat_buffer_view) },
            ],
            label: Some("ForwardGlobals.bind_group"),
        })
    }
    pub fn update(&mut self, world: &World, shadow_cameras: &[ShadowCameraData]) {
        let mut p = &mut self.params;
        if let Some(id) = get_active_camera(world, self.scene) {
            p.projection_view = world.get(id, projection_view()).unwrap_or_default();
            p.inv_projection_view = p.projection_view.inverse();
            p.camera_position = get_world_position(world, id).unwrap_or_default().extend(1.);
            p.camera_forward = world.get(id, local_to_world()).unwrap_or_default().transform_vector3(Vec3::Z);
            p.camera_far = world.get(id, far()).unwrap_or(1e3);
            p.fog = world.has_component(id, fog()) as i32;
            p.forward_camera_position = p.camera_position;
        }
        if let Some(sun) = get_active_sun(world, self.scene) {
            fn update<T, U>(out: &mut T, input: Result<U, ECSError>, mapper: impl Fn(U) -> T) {
                if let Ok(value) = input {
                    *out = mapper(value);
                }
            }

            update(&mut p.sun_direction, get_world_rotation(world, sun), |v| v.mul_vec3(Vec3::X).extend(1.));
            update(&mut p.sun_diffuse, world.get(sun, light_diffuse()), |v| v.extend(1.));
            update(&mut p.sun_ambient, world.get(sun, light_ambient()), |v| v.extend(1.));
            update(&mut p.fog_color, world.get(sun, fog_color()), |v| v.extend(1.));
            update(&mut p.fog_height_falloff, world.get(sun, fog_height_falloff()), |v| v);
            update(&mut p.fog_density, world.get(sun, fog_density()), |v| v);
        }
        self.params.time = Instant::now().duration_since(self.start_time).as_secs_f32();
        self.gpu.queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.params]));
        self.gpu.queue.write_buffer(&self.shadow_cameras_buffer, 0, bytemuck::cast_slice(shadow_cameras));
    }
}

fn create_dummy_shadow_texture(gpu: Arc<Gpu>) -> Arc<Texture> {
    Arc::new(Texture::new(
        gpu,
        &wgpu::TextureDescriptor {
            label: Some("ShadowGlobals.shadow_texture"),
            size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 2 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        },
    ))
}

pub struct ShadowAndUIGlobals {
    gpu: Arc<Gpu>,
    pub bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
}
impl ShadowAndUIGlobals {
    pub fn new(assets: AssetCache, layout: Arc<wgpu::BindGroupLayout>) -> Self {
        let gpu = GpuKey.get(&assets);
        let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ShadowGlobals.buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: std::mem::size_of::<GlobalParams>() as u64,
            mapped_at_creation: false,
        });
        let shadow_cameras_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ShadowGlobals.shadow_cameras_buffer"),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            size: std::mem::size_of::<Mat4>() as u64,
            mapped_at_creation: false,
        });
        let shadow_sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ShadowGlobals.shadow_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });
        let shadow_texture = create_dummy_shadow_texture(gpu.clone());
        let dummy_prev_frame = RenderTarget::new(gpu.clone(), UVec2::ONE, None);
        let shadow_view = shadow_texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            bind_group: gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::Sampler(&DefaultSamplerKey.get(&assets)) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()) },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Buffer(shadow_cameras_buffer.as_entire_buffer_binding()),
                    },
                    wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&shadow_sampler) },
                    wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&shadow_view) },
                    wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::TextureView(&dummy_prev_frame.color_buffer_view) },
                    wgpu::BindGroupEntry { binding: 6, resource: wgpu::BindingResource::TextureView(&dummy_prev_frame.depth_buffer_view) },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: wgpu::BindingResource::TextureView(&dummy_prev_frame.normals_quat_buffer_view),
                    },
                ],
                label: Some("ShadowGlobals.bind_group"),
            }),
            buffer,
            gpu,
        }
    }
    pub fn update(&self, world: &World, scene: Component<()>, projection_view: Mat4) {
        let mut params = GlobalParams {
            projection_view,
            camera_position: projection_view.inverse().project_point3(-Vec3::Z).extend(1.),
            ..Default::default()
        };
        if let Some(id) = get_active_camera(world, scene) {
            params.forward_camera_position = get_world_position(world, id).unwrap_or_default().extend(1.);
        }
        self.gpu.queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[params]));
    }
}
