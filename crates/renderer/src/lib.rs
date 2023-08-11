use core::fmt;
use std::{
    f32::consts::PI,
    fmt::{Debug, Formatter},
    str::FromStr,
    sync::Arc,
};

use ambient_core::{
    asset_cache, async_ecs::async_run, mesh, runtime, transform::get_world_rotation,
};
use ambient_ecs::{
    components, copy_component_recursive, query_mut, Debuggable, Entity, EntityId, Resource,
    SystemGroup, World,
};
use ambient_gpu::{
    gpu::Gpu,
    mesh_buffer::GpuMesh,
    shader_module::{BindGroupDesc, Shader, ShaderIdent, ShaderModule},
    wgsl_utils::wgsl_interpolate,
};
use ambient_gpu_ecs::{
    gpu_components, ComponentToGpuSystem, GpuComponentFormat, GpuWorldShaderModuleKey,
    GpuWorldSyncEvent,
};
use ambient_native_std::{asset_cache::*, asset_url::AbsAssetUrl, cb, include_file, Cb};
use derive_more::*;
use downcast_rs::{impl_downcast, DowncastSync};
use glam::{uvec4, UVec2, UVec4, Vec3};
use serde::{Deserialize, Serialize};

pub mod bind_groups;
mod collect;
mod culling;
mod globals;
pub mod lod;
pub mod materials;
mod outlines;
mod overlay_renderer;
mod renderer;
mod shaders;
mod shadow_renderer;
pub mod skinning;
mod target;
mod transparent_renderer;
mod tree_renderer;
use ambient_ecs::{query, Component};
pub use collect::*;
pub use culling::*;
pub use globals::*;
use materials::pbr_material::PbrMaterialFromUrl;
pub use materials::*;
use ordered_float::OrderedFloat;
pub use outlines::*;
pub use renderer::*;
pub use shaders::*;
pub use shadow_renderer::*;
pub use target::*;
pub use transparent_renderer::*;
pub use tree_renderer::*;

pub const MAX_PRIMITIVE_COUNT: usize = 16;

pub use ambient_ecs::generated::rendering::components::{
    cast_shadows, color, double_sided, fog_color, fog_density, fog_height_falloff, light_ambient,
    light_diffuse, overlay, pbr_material_from_url, scissors, scissors_recursive, sun,
    transparency_group,
};

components!("rendering", {
    @[Debuggable]
    primitives: Vec<RenderPrimitive>,

    /// The (cpu) primitives are split into an SoA on the gpu side
    @[Debuggable]
    gpu_primitives_mesh: [u32; MAX_PRIMITIVE_COUNT],
    @[Debuggable]
    gpu_primitives_lod: [u32; MAX_PRIMITIVE_COUNT],

    renderer_shader: RendererShaderProducer,
    material: SharedMaterial,
    @[Resource]
    renderer_stats: String,
});

gpu_components! {
    color() => color: GpuComponentFormat::Vec4,
    gpu_primitives_mesh() => gpu_primitives_mesh: GpuComponentFormat::Mat4,
    gpu_primitives_lod() => gpu_primitives_lod: GpuComponentFormat::Mat4,
}

pub fn init_all_components() {
    init_components();
    init_gpu_components();
    outlines::init_gpu_components();
    culling::init_gpu_components();
    lod::init_components();
    lod::init_gpu_components();
    skinning::init_components();
    skinning::init_gpu_components();
}

pub fn systems() -> SystemGroup {
    SystemGroup::new(
        "renderer",
        vec![
            Box::new(copy_component_recursive(
                "scissors",
                scissors_recursive(),
                scissors(),
            )),
            query(pbr_material_from_url().changed()).to_system(|q, world, qs, _| {
                for (id, url) in q.collect_cloned(world, qs) {
                    let url = match AbsAssetUrl::from_str(&url) {
                        Ok(value) => value,
                        Err(err) => {
                            log::warn!("Failed to parse pbr_material_from_url url: {:?}", err);
                            continue;
                        }
                    };
                    let assets = world.resource(asset_cache()).clone();
                    let async_run = world.resource(async_run()).clone();
                    world.resource(runtime()).spawn(async move {
                        match PbrMaterialFromUrl(url).get(&assets).await {
                            Err(err) => {
                                log::warn!("Failed to load pbr material from url: {:?}", err);
                            }
                            Ok(mat) => {
                                async_run.run(move |world| {
                                    world
                                        .add_components(
                                            id,
                                            Entity::new()
                                                .with(
                                                    renderer_shader(),
                                                    cb(pbr_material::get_pbr_shader),
                                                )
                                                .with(material(), mat.into()),
                                        )
                                        .ok();
                                });
                            }
                        }
                    });
                }
            }),
            query_mut(
                (primitives(),),
                (
                    renderer_shader().changed(),
                    material().changed(),
                    mesh().changed(),
                ),
            )
            .to_system(|q, world, qs, _| {
                for (_, (primitives,), (shader, material, mesh)) in q.iter(world, qs) {
                    *primitives = vec![RenderPrimitive {
                        shader: shader.clone(),
                        material: material.clone(),
                        mesh: mesh.clone(),
                        lod: 0,
                    }];
                }
            }),
            query_mut(
                (gpu_primitives_mesh(), gpu_primitives_lod()),
                (primitives().changed(),),
            )
            .to_system(|q, world, qs, _| {
                for (id, (p_mesh, p_lod), (primitives,)) in q.iter(world, qs) {
                    if primitives.len() > MAX_PRIMITIVE_COUNT {
                        log::warn!(
                            "Entity {} has more than {MAX_PRIMITIVE_COUNT} primitives",
                            id
                        );
                    }
                    for (i, p) in primitives.iter().enumerate().take(MAX_PRIMITIVE_COUNT) {
                        p_mesh[i] = p.mesh.index();
                        p_lod[i] = p.lod as u32;
                    }
                }
            }),
            Box::new(outlines::systems()),
        ],
    )
}

pub fn gpu_world_systems(gpu: Arc<Gpu>) -> SystemGroup<GpuWorldSyncEvent> {
    SystemGroup::new(
        "renderer/gpu_world_update",
        vec![
            Box::new(outlines::gpu_world_systems(gpu.clone())),
            Box::new(ComponentToGpuSystem::new(
                gpu.clone(),
                GpuComponentFormat::Vec4,
                color(),
                gpu_components::color(),
            )),
            Box::new(ComponentToGpuSystem::new(
                gpu.clone(),
                GpuComponentFormat::Mat4,
                gpu_primitives_mesh(),
                gpu_components::gpu_primitives_mesh(),
            )),
            Box::new(ComponentToGpuSystem::new(
                gpu.clone(),
                GpuComponentFormat::Mat4,
                gpu_primitives_lod(),
                gpu_components::gpu_primitives_lod(),
            )),
            Box::new(lod::gpu_world_system(gpu.clone())),
            Box::new(skinning::gpu_world_systems(gpu)),
        ],
    )
}

pub fn get_active_sun(world: &World, scene: Component<()>) -> Option<EntityId> {
    query((scene, sun()))
        .iter(world, None)
        .max_by_key(|(_, (_, x))| OrderedFloat(**x))
        .map(|(id, _)| id)
}
pub fn get_sun_light_direction(world: &World, scene: Component<()>) -> Vec3 {
    get_active_sun(world, scene)
        .and_then(|sun| get_world_rotation(world, sun).ok())
        .map(|rot| rot.mul_vec3(Vec3::X))
        .unwrap_or(default_sun_direction())
}

#[derive(Clone, Debug)]
pub struct RenderPrimitive {
    pub material: SharedMaterial,
    pub shader: RendererShaderProducer,
    pub mesh: Arc<GpuMesh>,
    pub lod: usize,
}

pub type PrimitiveIndex = usize;

pub fn get_gpu_primitive_id(
    world: &World,
    id: EntityId,
    primitive_index: PrimitiveIndex,
    material_index: u32,
) -> UVec4 {
    let loc = world.entity_loc(id).unwrap();
    uvec4(
        loc.archetype as u32,
        loc.index as u32,
        primitive_index as u32,
        material_index,
    )
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuRenderPrimitive {
    pub mesh: u32,
    pub lod: u32,
    pub _padding: UVec2,
}

#[derive(Clone, Deref, DerefMut)]
pub struct SharedMaterial(pub Arc<dyn Material + 'static>);

impl Debug for SharedMaterial {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&*self.0, f)
    }
}

impl<T: Material + 'static> From<Arc<T>> for SharedMaterial {
    fn from(v: Arc<T>) -> Self {
        Self(v as Arc<dyn Material>)
    }
}

impl SharedMaterial {
    pub fn replace(&mut self, value: impl Material + 'static) {
        self.0 = Arc::new(value)
    }

    pub fn new(value: impl Material + 'static) -> Self {
        Self(Arc::new(value))
    }

    pub fn borrow_downcast<T: Material>(&self) -> &T {
        self.0.downcast_ref::<T>().unwrap()
    }
}

impl<T> From<T> for SharedMaterial
where
    T: Material + 'static,
{
    fn from(v: T) -> Self {
        Self::new(v)
    }
}

/// No bind groups
pub fn get_defs_module() -> Arc<ShaderModule> {
    let iter = [("PI", PI)].iter();
    #[cfg(not(target_os = "unknown"))]
    let iter = iter.map(|(k, v)| format!("const {k}: f32 = {v};\n"));
    #[cfg(target_os = "unknown")]
    let iter = iter.map(|(k, v)| format!("const {k}: f32 = {v};\n"));

    let iter = iter
        .chain([wgsl_interpolate(), include_file!("polyfill.wgsl")])
        .collect::<String>();

    Arc::new(ShaderModule::new("defs", iter))
}

pub fn get_mesh_meta_module(bind_group_offset: u32) -> Arc<ShaderModule> {
    Arc::new(
        ShaderModule::new("mesh_meta", include_file!("mesh_meta.wgsl"))
            .with_ident(ShaderIdent::constant(
                "MESH_METADATA_BINDING",
                bind_group_offset + MESH_METADATA_BINDING,
            ))
            .with_binding_desc(get_mesh_meta_layout(bind_group_offset)),
    )
}

pub fn get_mesh_data_module(bind_group_offset: u32) -> Arc<ShaderModule> {
    Arc::new(
        ShaderModule::new("mesh_data", include_file!("mesh_data.wgsl"))
            .with_ident(ShaderIdent::constant(
                "MESH_BASE_BINDING",
                bind_group_offset + MESH_BASE_BINDING,
            ))
            .with_ident(ShaderIdent::constant(
                "MESH_SKIN_BINDING",
                bind_group_offset + MESH_SKIN_BINDING,
            ))
            .with_ident(ShaderIdent::constant(
                "SKINS_BINDING",
                bind_group_offset + SKINS_BINDING,
            ))
            .with_binding_desc(get_mesh_data_layout(bind_group_offset))
            .with_dependency(get_mesh_meta_module(bind_group_offset)),
    )
}

pub fn primitives_layout() -> BindGroupDesc<'static> {
    BindGroupDesc {
        entries: vec![wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: PRIMITIVES_BIND_GROUP.into(),
    }
}

pub fn get_common_layout() -> BindGroupDesc<'static> {
    BindGroupDesc {
        entries: vec![wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: PRIMITIVES_BIND_GROUP.into(),
    }
}

/// Includes entity locs
pub fn get_common_module(_: &AssetCache) -> Arc<ShaderModule> {
    Arc::new(
        ShaderModule::new("renderer_common", include_file!("renderer_common.wgsl"))
            .with_binding_desc(get_common_layout())
            .with_dependency(get_mesh_data_module(GLOBALS_BIND_GROUP_SIZE)),
    )
}

/// Contains scene globals and shadow maps
pub fn get_globals_module(_assets: &AssetCache, shadow_cascades: u32) -> Arc<ShaderModule> {
    Arc::new(
        ShaderModule::new("globals", include_file!("globals.wgsl"))
            .with_ident(ShaderIdent::constant("SHADOW_CASCADES", shadow_cascades))
            .with_binding_desc(globals_layout()),
    )
}

pub fn get_forward_modules(assets: &AssetCache, shadow_cascades: u32) -> Vec<Arc<ShaderModule>> {
    vec![
        get_defs_module(),
        get_mesh_data_module(GLOBALS_BIND_GROUP_SIZE),
        get_globals_module(assets, shadow_cascades),
        GpuWorldShaderModuleKey { read_only: true }.get(assets),
        get_common_module(assets),
    ]
}

pub fn get_overlay_modules(assets: &AssetCache, shadow_cascades: u32) -> Vec<Arc<ShaderModule>> {
    vec![
        get_defs_module(),
        get_globals_module(assets, shadow_cascades),
        get_mesh_data_module(GLOBALS_BIND_GROUP_SIZE),
    ]
}

pub struct MaterialShader {
    pub id: String,
    pub shader: Arc<ShaderModule>,
}

pub trait Material: Debug + Sync + Send + DowncastSync {
    fn id(&self) -> &str;

    fn name(&self) -> &str {
        self.id()
    }

    fn update(&self, _: &Gpu, _: &World) {}

    fn bind_group(&self) -> &wgpu::BindGroup;

    fn transparent(&self) -> Option<bool> {
        None
    }

    fn double_sided(&self) -> Option<bool> {
        None
    }
    /// TODO: Apply to tree renderer too (only applies to transparent now)
    fn depth_write_enabled(&self) -> Option<bool> {
        None
    }
    fn transparency_group(&self) -> Option<i32> {
        None
    }
}

impl_downcast!(sync Material);

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[repr(u32)]
pub enum AlphaMode {
    Opaque,
    Mask,
    Blend,
}
impl Default for AlphaMode {
    fn default() -> Self {
        Self::Opaque
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FSMain {
    Forward,
    Shadow,
    Outline,
}

pub struct RendererShader {
    pub id: String,
    pub shader: Arc<Shader>,
    pub vs_main: String,
    pub fs_shadow_main: String,
    pub fs_forward_main: String,
    pub fs_outline_main: String,
    pub transparent: bool,
    pub double_sided: bool,
    /// TODO: Apply to tree renderer too (only applies to transparent now)
    pub depth_write_enabled: bool,
    pub transparency_group: i32,
}
impl RendererShader {
    fn get_fs_main_name(&self, main: FSMain) -> &str {
        match main {
            FSMain::Forward => &self.fs_forward_main,
            FSMain::Shadow => &self.fs_shadow_main,
            FSMain::Outline => &self.fs_outline_main,
        }
    }
}
impl Debug for RendererShader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RendererShader")
            .field("id", &self.id)
            .finish()
    }
}

pub type RendererShaderProducer =
    Cb<dyn Fn(&AssetCache, &RendererConfig) -> Arc<RendererShader> + Sync + Send>;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DrawIndexedIndirect {
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    base_vertex: u32,
    first_instance: u32,
}

fn is_transparent(
    world: &World,
    id: EntityId,
    material: &SharedMaterial,
    shader: &RendererShader,
) -> bool {
    world.get(id, transparency_group()).is_ok()
        || material.transparent().unwrap_or(shader.transparent)
}

/// wgpu will throw an exception if the scissor value is outside the viewport
pub(crate) fn set_scissors_safe(
    render_pass: &mut wgpu::RenderPass,
    render_target_size: wgpu::Extent3d,
    scissors: Option<UVec4>,
) -> bool {
    if let Some(scissors) = scissors {
        let left = scissors.x.clamp(0, render_target_size.width);
        let top = scissors.y.clamp(0, render_target_size.height);
        let right = (left + scissors.z).clamp(0, render_target_size.width);
        let bottom = (top + scissors.w).clamp(0, render_target_size.height);
        let width = right - left;
        let height = bottom - top;
        if width > 0 && height > 0 {
            render_pass.set_scissor_rect(left, top, width, height);
            true
        } else {
            false
        }
    } else {
        render_pass.set_scissor_rect(0, 0, render_target_size.width, render_target_size.height);
        true
    }
}
