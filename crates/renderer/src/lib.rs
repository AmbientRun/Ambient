use std::{f32::consts::PI, sync::Arc};

use derive_more::*;
use downcast_rs::{impl_downcast, DowncastSync};
use glam::{uvec4, UVec2, UVec4, Vec3, Vec4};
use kiwi_core::{
    gpu_components,
    gpu_ecs::{ComponentToGpuSystem, GpuComponentFormat, GpuWorldShaderModuleKey, GpuWorldSyncEvent},
    mesh,
    transform::get_world_rotation,
};
use kiwi_ecs::{
    components, query_mut, Debuggable, Description, EntityId, MakeDefault, Name, Networked, Resource, Store, SystemGroup, World,
};
use kiwi_gpu::{
    mesh_buffer::{get_mesh_buffer_types, GpuMesh, MESH_BUFFER_TYPES_WGSL},
    shader_module::{BindGroupDesc, Shader, ShaderModule, ShaderModuleIdentifier},
    wgsl_utils::wgsl_interpolate,
};
use kiwi_std::{asset_cache::*, include_file, Cb};
use serde::{Deserialize, Serialize};

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
pub use collect::*;
pub use culling::*;
pub use globals::*;
use kiwi_ecs::{query, Component};
pub use materials::*;
use ordered_float::OrderedFloat;
pub use outlines::*;
pub use renderer::*;
pub use shaders::*;
pub use shadow_renderer::*;
pub use target::*;
pub use transparent_renderer::*;
pub use tree_renderer::*;

pub const MAX_PRIMITIVE_COUNT: usize = 20;

components!("rendering", {
    @[Debuggable]
    primitives: Vec<RenderPrimitive>,
    gpu_primitives: [GpuRenderPrimitive; MAX_PRIMITIVE_COUNT],
    renderer_shader: RendererShaderProducer,
    material: SharedMaterial,
    @[Resource]
    renderer_stats: String,
    @[
        MakeDefault, Debuggable, Networked, Store,
        Name["Overlay"],
        Description["If attached, this entity will be rendered with an overlay."]
    ]
    overlay: (),
    @[
        MakeDefault, Debuggable, Networked, Store,
        Name["Color"],
        Description["This entity will be tinted with the specified color if the color is not black."]
    ]
    color: Vec4,
    @[
        MakeDefault, Debuggable, Networked, Store,
        Name["Double-sided"],
        Description["If this is set, the entity will be rendered with double-sided rendering."]
    ]
    double_sided: bool,
    @[
        MakeDefault, Debuggable, Networked, Store,
        Name["Cast shadows"],
        Description["If attached, this entity will cast shadows."]
    ]
    cast_shadows: (),
    @[
        Debuggable, Networked, Store,
        Name["Sun"],
        Description["Marks this entity as a sun (i.e. its rotation will be used to control the global light direction).\nThe entity with the highest `sun` value takes precedence."]
    ]
    sun: f32,
    @[
        Debuggable, Networked, Store,
        Name["Light diffuse"],
        Description["The diffuse light color of the `sun`."]
    ]
    light_diffuse: Vec3,
    @[
        Debuggable, Networked, Store,
        Name["Light ambient"],
        Description["The ambient light color of the `sun`."]
    ]
    light_ambient: Vec3,
    @[
        Debuggable, Networked, Store,
        Name["Fog color"],
        Description["The color of the fog for this `sun`."]
    ]
    fog_color: Vec3,
    @[
        Debuggable, Networked, Store,
        Name["Fog height fall-off"],
        Description["The height at which the fog will fall off (i.e. stop being visible) for this `sun`."]
    ]
    fog_height_falloff: f32,
    @[
        Debuggable, Networked, Store,
        Name["Fog density"],
        Description["The density of the fog for this `sun`."]
    ]
    fog_density: f32,
    @[
        Debuggable, Networked, Store,
        Name["Transparency group"],
        Description["Controls when this transparent object will be rendered. Transparent objects are sorted by (transparency_group, z-depth)."]
    ]
    transparency_group: i32,
});
gpu_components! {
    color() => color: GpuComponentFormat::Vec4,
    primitives() => primitives: GpuComponentFormat::UVec4Array20,
}
pub fn init_all_componets() {
    init_components();
    init_gpu_components();
    outlines::init_components();
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
            query_mut((primitives(),), (renderer_shader().changed(), material().changed(), mesh().changed())).to_system(
                |q, world, qs, _| {
                    for (_, (primitives,), (shader, material, mesh)) in q.iter(world, qs) {
                        *primitives =
                            vec![RenderPrimitive { shader: shader.clone(), material: material.clone(), mesh: mesh.clone(), lod: 0 }];
                    }
                },
            ),
            query_mut((gpu_primitives(),), (primitives().changed(),)).to_system(|q, world, qs, _| {
                for (id, (gpu_primitives,), (primitives,)) in q.iter(world, qs) {
                    if primitives.len() > MAX_PRIMITIVE_COUNT {
                        log::warn!("Entity {} has more than {MAX_PRIMITIVE_COUNT} primitives", id);
                    }
                    for (i, p) in primitives.iter().enumerate().take(MAX_PRIMITIVE_COUNT) {
                        gpu_primitives[i].mesh = p.mesh.index() as u32;
                        gpu_primitives[i].lod = p.lod as u32;
                    }
                }
            }),
            Box::new(outlines::systems()),
        ],
    )
}

pub fn gpu_world_systems() -> SystemGroup<GpuWorldSyncEvent> {
    SystemGroup::new(
        "renderer/gpu_world_update",
        vec![
            Box::new(outlines::gpu_world_systems()),
            Box::new(ComponentToGpuSystem::new(GpuComponentFormat::Vec4, color(), gpu_components::color())),
            Box::new(ComponentToGpuSystem::new(GpuComponentFormat::UVec4Array20, gpu_primitives(), gpu_components::primitives())),
            Box::new(lod::gpu_world_system()),
            Box::new(skinning::gpu_world_systems()),
        ],
    )
}

pub fn get_active_sun(world: &World, scene: Component<()>) -> Option<EntityId> {
    query((scene, sun())).iter(world, None).max_by_key(|(_, (_, x))| OrderedFloat(**x)).map(|(id, _)| id)
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
pub fn get_gpu_primitive_id(world: &World, id: EntityId, primitive_index: PrimitiveIndex, material_index: u32) -> UVec4 {
    let loc = world.entity_loc(id).unwrap();
    uvec4(loc.archetype as u32, loc.index as u32, primitive_index as u32, material_index)
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuRenderPrimitive {
    pub mesh: u32,
    pub lod: u32,
    pub _padding: UVec2,
}

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct SharedMaterial(Arc<dyn Material + 'static>);

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
pub fn get_defs_module(_: &AssetCache) -> ShaderModule {
    ShaderModule::from_str(
        "Definitions",
        [("PI", PI)]
            .iter()
            .map(|(k, v)| format!("let {k}: f32 = {v};\n"))
            .chain([wgsl_interpolate(), include_file!("polyfill.wgsl"), MESH_BUFFER_TYPES_WGSL.to_string()])
            .collect::<String>(),
    )
}

pub fn get_resources_module() -> ShaderModule {
    let idents = vec![
        ShaderModuleIdentifier::constant("MESH_METADATA_BINDING", MESH_METADATA_BINDING),
        ShaderModuleIdentifier::constant("MESH_NORMAL_BINDING", MESH_NORMAL_BINDING),
        ShaderModuleIdentifier::constant("MESH_TANGENT_BINDING", MESH_TANGENT_BINDING),
        ShaderModuleIdentifier::constant("MESH_POSITION_BINDING", MESH_POSITION_BINDING),
        ShaderModuleIdentifier::constant("MESH_TEXCOORD0_BINDING", MESH_TEXCOORD0_BINDING),
        ShaderModuleIdentifier::constant("MESH_JOINT_BINDING", MESH_JOINT_BINDING),
        ShaderModuleIdentifier::constant("MESH_WEIGHT_BINDING", MESH_WEIGHT_BINDING),
        ShaderModuleIdentifier::constant("SKINS_BINDING", SKINS_BINDING),
        ShaderModuleIdentifier::bind_group(get_resources_layout()),
    ];

    ShaderModule::new("Resources", include_file!("resources.wgsl"), idents)
}

pub fn primitives_layout() -> BindGroupDesc {
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
pub fn get_common_module(_: &AssetCache) -> ShaderModule {
    // let primtives_layout = ;
    ShaderModule::new("Common", include_file!("renderer_common.wgsl"), vec![primitives_layout().into()])
}

/// Contains scene globals and shadow maps
pub fn get_globals_module(assets: &AssetCache, shadow_cascades: u32) -> ShaderModule {
    ShaderModule::new(
        "Globals",
        include_file!("globals.wgsl"),
        vec![ShaderModuleIdentifier::constant("SHADOW_CASCADES", shadow_cascades), globals_layout().into()],
    )
}

pub fn get_forward_module(assets: &AssetCache, shadow_cascades: u32) -> ShaderModule {
    [
        get_defs_module(assets),
        get_resources_module(),
        get_globals_module(assets, shadow_cascades),
        GpuWorldShaderModuleKey { read_only: true }.get(assets),
        get_common_module(assets),
        get_mesh_buffer_types(),
    ]
    .iter()
    .collect()
}

pub fn get_overlay_module(assets: &AssetCache, shadow_cascades: u32) -> ShaderModule {
    [get_defs_module(assets), get_globals_module(assets, shadow_cascades)].iter().collect()
}

pub struct MaterialShader {
    pub id: String,
    pub shader: ShaderModule,
}

pub trait Material: std::fmt::Debug + Sync + Send + DowncastSync {
    fn id(&self) -> &str;
    fn name(&self) -> &str {
        self.id()
    }
    fn update(&self, _: &World) {}
    fn bind(&self) -> &wgpu::BindGroup;
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
    pub fn material_layout(&self) -> &Arc<wgpu::BindGroupLayout> {
        self.shader.get_bind_group_layout_by_name(MATERIAL_BIND_GROUP).unwrap()
    }
    fn get_fs_main_name(&self, main: FSMain) -> &str {
        match main {
            FSMain::Forward => &self.fs_forward_main,
            FSMain::Shadow => &self.fs_shadow_main,
            FSMain::Outline => &self.fs_outline_main,
        }
    }
}
impl std::fmt::Debug for RendererShader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RendererShader").field("id", &self.id).finish()
    }
}

pub type RendererShaderProducer = Cb<dyn Fn(&AssetCache, &RendererConfig) -> Arc<RendererShader> + Sync + Send>;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DrawIndexedIndirect {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub base_index: u32,
    pub vertex_offset: i32,
    pub base_instance: u32,
}
