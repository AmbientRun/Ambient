use std::{str::FromStr, sync::Arc};

use ambient_core::{
    asset_cache,
    async_ecs::async_run,
    mesh, runtime,
    transform::{local_to_world, mesh_to_local, mesh_to_world, rotation, scale, translation},
};
use ambient_ecs::{
    ensure_has_component, ensure_has_component_with_default, query, Entity, SystemGroup,
};
use ambient_gpu::{
    gpu::{Gpu, GpuKey},
    sampler::SamplerKey,
    shader_module::{BindGroupDesc, ShaderModule},
    std_assets::PixelTextureKey,
    texture::Texture,
    texture_loaders::TextureFromUrl,
    typed_buffer::TypedBuffer,
};
use ambient_layout::{gpu_ui_size, height, mesh_to_local_from_size, width};
use ambient_meshes::{UIRectMeshKey, UnitQuadMeshKey};
use ambient_renderer::{
    gpu_primitives_lod, gpu_primitives_mesh, material, primitives, renderer_shader, Material,
    MaterialShader, RendererConfig, RendererShader, SharedMaterial, StandardShaderKey,
    MATERIAL_BIND_GROUP,
};
use ambient_std::{
    asset_cache::{AssetCache, AsyncAssetKey, AsyncAssetKeyExt, SyncAssetKey, SyncAssetKeyExt},
    asset_url::AbsAssetUrl,
    cb,
    color::Color,
    download_asset::AssetResult,
    friendly_id, include_file,
};
use async_trait::async_trait;
use glam::{uvec4, vec4, Quat, UVec3, UVec4, Vec3, Vec4};
use wgpu::{BindGroup, BindGroupLayoutEntry, Extent3d};

pub use ambient_ecs::generated::components::core::rect::{
    background_color, background_url, border_color, border_radius, border_thickness, line_from,
    line_to, line_width, rect, size_from_background_image,
};

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Corners {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_left: f32,
    pub bottom_right: f32,
}
impl Corners {
    pub fn even(value: f32) -> Self {
        Self {
            top_left: value,
            top_right: value,
            bottom_left: value,
            bottom_right: value,
        }
    }
}
impl From<Corners> for Vec4 {
    fn from(value: Corners) -> Self {
        vec4(
            value.top_left,
            value.top_right,
            value.bottom_left,
            value.bottom_right,
        )
    }
}
impl From<Vec4> for Corners {
    fn from(value: Vec4) -> Self {
        Self {
            top_left: value.x,
            top_right: value.y,
            bottom_left: value.z,
            bottom_right: value.w,
        }
    }
}

pub fn systems() -> SystemGroup {
    SystemGroup::new(
        "ui/rect",
        vec![
            query((
                line_from().changed(),
                line_to().changed(),
                line_width().changed(),
            ))
            .to_system(|q, world, qs, _| {
                for (id, (from, to, line_width)) in q.collect_cloned(world, qs) {
                    // we need to handle the 180 degree rotation case
                    let dir = (to - from).normalize();
                    // no need to compare y
                    // it can be problematic even when it's not equal but closer than 0.001
                    // see:
                    // https://docs.rs/glam/latest/glam/f32/struct.Quat.html#method.from_rotation_arc
                    let rot = if from.x >= to.x {
                        Quat::from_rotation_arc(-Vec3::X, dir)
                    } else {
                        Quat::from_rotation_arc(Vec3::X, dir)
                    };
                    world
                        .add_components(
                            id,
                            Entity::new()
                                .with(translation(), (from + to) / 2.)
                                .with(rotation(), rot)
                                .with_default(rect())
                                .with(width(), (from - to).length())
                                .with(height(), line_width),
                        )
                        .unwrap();
                }
            }),
            ensure_has_component_with_default(rect(), primitives()),
            ensure_has_component_with_default(rect(), gpu_ui_size()),
            ensure_has_component_with_default(rect(), mesh_to_local()),
            ensure_has_component_with_default(rect(), mesh_to_world()),
            ensure_has_component_with_default(rect(), local_to_world()),
            ensure_has_component(rect(), scale(), Vec3::ONE),
            ensure_has_component_with_default(rect(), mesh_to_local_from_size()),
            ensure_has_component_with_default(rect(), gpu_primitives_mesh()),
            ensure_has_component_with_default(rect(), gpu_primitives_lod()),
            query(())
                .incl(rect())
                .excl(mesh())
                .to_system(|q, world, qs, _| {
                    let assets = world.resource(asset_cache()).clone();
                    for (id, _) in q.collect_cloned(world, qs) {
                        world
                            .add_components(
                                id,
                                Entity::new()
                                    .with(
                                        mesh(),
                                        if world.has_component(id, line_from()) {
                                            UnitQuadMeshKey.get(&assets)
                                        } else {
                                            UIRectMeshKey.get(&assets)
                                        },
                                    )
                                    .with(renderer_shader(), cb(get_rect_shader)),
                            )
                            .unwrap();
                    }
                }),
            query(())
                .incl(rect())
                .optional_changed(background_color())
                .optional_changed(background_url())
                .optional_changed(border_color())
                .optional_changed(border_radius())
                .optional_changed(border_thickness())
                .to_system(|q, world, qs, _| {
                    let runtime = world.resource(runtime()).clone();
                    for (id, _) in q.collect_cloned(world, qs) {
                        let assets = world.resource(asset_cache()).clone();
                        let async_run = world.resource(async_run()).clone();
                        let mat_key = RectMaterialKey {
                            params: RectMaterialParams {
                                background_color: world
                                    .get(id, background_color())
                                    .unwrap_or(Color::WHITE.into()),
                                border_color: world
                                    .get(id, border_color())
                                    .unwrap_or(Color::WHITE.into()),
                                border_radius: world
                                    .get(id, border_radius())
                                    .unwrap_or_default()
                                    .into(),
                                border_thickness: world.get(id, border_thickness()).unwrap_or(0.),
                                _padding: Default::default(),
                            },
                            background: world.get_cloned(id, background_url()).ok(),
                        };
                        let resize = world.has_component(id, size_from_background_image());
                        runtime.spawn(async move {
                            let mat = mat_key.get(&assets).await;
                            match mat {
                                Ok(mat) => {
                                    async_run.run(move |world| {
                                        world
                                            .add_component(
                                                id,
                                                material(),
                                                SharedMaterial(mat.clone()),
                                            )
                                            .ok();
                                        if resize {
                                            world
                                                .add_components(
                                                    id,
                                                    Entity::new()
                                                        .with(
                                                            width(),
                                                            mat.background_size.width as f32,
                                                        )
                                                        .with(
                                                            height(),
                                                            mat.background_size.height as f32,
                                                        ),
                                                )
                                                .ok();
                                        }
                                    });
                                }
                                Err(err) => {
                                    log::error!(
                                        "Failed to load material for entity {}: {:?}",
                                        id,
                                        err
                                    )
                                }
                            }
                        });
                    }
                }),
        ],
    )
}

#[derive(Debug)]
pub struct RectMaterialShaderKey;
impl SyncAssetKey<Arc<MaterialShader>> for RectMaterialShaderKey {
    fn load(&self, _assets: AssetCache) -> Arc<MaterialShader> {
        Arc::new(MaterialShader {
            shader: Arc::new(
                ShaderModule::new("RectMaterial", include_file!("rect.wgsl"))
                    .with_binding_desc(get_rect_layout()),
            ),

            id: "rect_material_shader".to_string(),
        })
    }
}

fn get_rect_layout() -> BindGroupDesc<'static> {
    BindGroupDesc {
        entries: vec![
            BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
        ],
        label: MATERIAL_BIND_GROUP.into(),
    }
}

pub fn get_rect_shader(assets: &AssetCache, config: &RendererConfig) -> Arc<RendererShader> {
    StandardShaderKey {
        material_shader: RectMaterialShaderKey.get(assets),
        lit: false,
        shadow_cascades: config.shadow_cascades,
    }
    .get(assets)
}

#[derive(Debug, Clone)]
struct RectMaterialKey {
    pub params: RectMaterialParams,
    pub background: Option<String>,
}
#[async_trait]
impl AsyncAssetKey<AssetResult<Arc<RectMaterial>>> for RectMaterialKey {
    async fn load(self, assets: AssetCache) -> AssetResult<Arc<RectMaterial>> {
        let gpu = GpuKey.get(&assets);
        let error_color = PixelTextureKey {
            colors: vec![uvec4(255, 0, 0, 255)],
        };
        let background = match self.background {
            Some(url) => match AbsAssetUrl::from_str(&url) {
                Ok(url) => {
                    let tex = TextureFromUrl {
                        url: url.clone(),
                        format: wgpu::TextureFormat::Rgba8Unorm,
                    };
                    match tex.get(&assets).await {
                        Ok(texture) => texture,
                        Err(err) => {
                            log::warn!("Failed to load image at url {}: {:?}", url, err);
                            error_color.get(&assets)
                        }
                    }
                }
                Err(err) => {
                    log::warn!("Failed to load image at url {}: {:?}", url, err);
                    error_color.get(&assets)
                }
            },
            None => PixelTextureKey {
                colors: vec![UVec4::ZERO],
            }
            .get(&assets),
        };
        Ok(Arc::new(RectMaterial::new(
            &gpu,
            &assets,
            self.params,
            &background,
        )))
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RectMaterialParams {
    pub background_color: Vec4,
    pub border_color: Vec4,
    pub border_radius: Corners,
    pub border_thickness: f32,
    pub _padding: UVec3,
}

pub struct RectMaterial {
    id: String,
    bind_group: wgpu::BindGroup,
    transparent: Option<bool>,
    background_size: Extent3d,
}
impl RectMaterial {
    pub fn new(
        gpu: &Gpu,
        assets: &AssetCache,
        params: RectMaterialParams,
        background: &Texture,
    ) -> Self {
        let layout = get_rect_layout().get(assets);

        let buffer = TypedBuffer::new_init(
            gpu,
            "RectMaterial.buffer",
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            &[params],
        );
        let sampler = SamplerKey::LINEAR_CLAMP_TO_EDGE.get(assets);

        Self {
            id: friendly_id(),
            bind_group: gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(
                            buffer.buffer().as_entire_buffer_binding(),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(
                            &background.handle.create_view(&Default::default()),
                        ),
                    },
                ],
                label: Some("RectMaterial.bind_group"),
            }),
            transparent: Some(true),
            background_size: background.size,
        }
    }
}

impl std::fmt::Debug for RectMaterial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RectMaterial")
            .field("id", &self.id)
            .finish()
    }
}

impl Material for RectMaterial {
    fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn transparent(&self) -> Option<bool> {
        self.transparent
    }
}
