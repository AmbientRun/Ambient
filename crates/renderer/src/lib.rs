use std::{f32::consts::PI, fmt::Debug, sync::Arc};

use ambient_core::{
    asset_cache,
    async_ecs::async_run,
    bounding::{local_bounding_aabb, world_bounding_aabb, world_bounding_sphere},
    gpu_components,
    gpu_ecs::{
        ComponentToGpuSystem, GpuComponentFormat, GpuWorldShaderModuleKey, GpuWorldSyncEvent,
    },
    main_scene, mesh, runtime,
    transform::{get_world_rotation, local_to_world, mesh_to_world},
};
use ambient_ecs::{
    components,
    generated::{
        components::core::rendering::{material_from_url, mesh_from_url},
        concepts::make_transformable,
    },
    query_mut, Debuggable, Entity, EntityId, Resource, SystemGroup, World,
};
use ambient_gpu::{
    mesh_buffer::GpuMesh,
    shader_module::{BindGroupDesc, Shader, ShaderIdent, ShaderModule},
    wgsl_utils::wgsl_interpolate,
};
use ambient_std::{asset_cache::*, asset_url::AbsAssetUrl, cb, include_file, mesh::MeshKey, Cb};
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
use materials::pbr_material::{PbrMaterialConfigKey, PbrMaterialFromUrl};
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

pub use ambient_ecs::generated::components::core::rendering::{
    cast_shadows, color, double_sided, fog_color, fog_density, fog_height_falloff, light_ambient,
    light_diffuse, overlay, pbr_material_from_url, sun, transparency_group,
};

use crate::pbr_material::{get_pbr_shader, PbrMaterial};

components!("rendering", {
    @[Debuggable]
    primitives: Vec<RenderPrimitive>,

    /// The (cpu) primitives are split into an SoA on the gpu side
    gpu_primitives_mesh: [u32; MAX_PRIMITIVE_COUNT],
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
            query(pbr_material_from_url().changed()).to_system(|q, world, qs, _| {
                for (id, url) in q.collect_cloned(world, qs) {
                    let url = match AbsAssetUrl::parse(url) {
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
                        p_mesh[i] = p.mesh.index() as u32;
                        p_lod[i] = p.lod as u32;
                    }
                }
            }),
            Box::new(outlines::systems()),
            query(mesh_from_url().changed()).to_system(|q, world, qs, _| {
                // todo: tidy all this up, move material to guest side, ensure_has_component
                let assets = world.resource(asset_cache()).clone();
                for (id, mesh_url) in q.collect_cloned(world, qs) {
                    use std::str::FromStr;
                    let Ok(mesh_key) = MeshKey::from_str(&mesh_url) else {
                        tracing::warn!("Invalid mesh url, must be a valid ULID");
                        return;
                    };
                    let Some(mesh) = mesh_key.try_get(&assets) else {
                        tracing::warn!("Could not find mesh from {mesh_key}");
                        return;
                    };

                    world
                        .add_components(
                            id,
                            Entity::new()
                                .with(ambient_core::mesh(), GpuMesh::from_mesh(&assets, &mesh))
                                .with_default(main_scene())
                                .with_default(gpu_primitives_mesh())
                                .with_default(gpu_primitives_lod())
                                .with_default(primitives())
                                .with_default(local_to_world())
                                .with_default(mesh_to_world())
                                .with_merge(make_transformable())
                                .with(local_bounding_aabb(), mesh.aabb())
                                .with(world_bounding_aabb(), mesh.aabb())
                                .with(world_bounding_sphere(), mesh.aabb().to_sphere()),
                        )
                        .unwrap();
                }
            }),
            query(material_from_url().changed()).to_system(|q, world, qs, _| {
                let assets = world.resource(asset_cache()).clone();
                for (id, material_url) in q.collect_cloned(world, qs) {
                    use std::str::FromStr;
                    let Some(material_url) = material_url.strip_prefix("ambient-asset-transient:/") else {
                        tracing::warn!("Invalid material url, must start with ambient-asset-transient:/");
                        return;
                    };
                    let Ok(material_key) = PbrMaterialConfigKey::from_str(&material_url) else {
                        tracing::warn!("Invalid material url, must be a valid ULID");
                        return;
                    };
                    let Some(material) = material_key.try_get(&assets) else {
                        tracing::warn!("Could not find material from {material_key}");
                        return;
                    };
                    let material = PbrMaterial::new(&assets, material);
                    let material = SharedMaterial::new(material);
                    world
                        .add_components(
                            id,
                            Entity::new()
                                .with(crate::material(), material)
                                .with(renderer_shader(), cb(get_pbr_shader)),
                        )
                        .unwrap();
                }

            })
        ],
    )
}

pub fn gpu_world_systems() -> SystemGroup<GpuWorldSyncEvent> {
    SystemGroup::new(
        "renderer/gpu_world_update",
        vec![
            Box::new(outlines::gpu_world_systems()),
            Box::new(ComponentToGpuSystem::new(
                GpuComponentFormat::Vec4,
                color(),
                gpu_components::color(),
            )),
            Box::new(ComponentToGpuSystem::new(
                GpuComponentFormat::Mat4,
                gpu_primitives_mesh(),
                gpu_components::gpu_primitives_mesh(),
            )),
            Box::new(ComponentToGpuSystem::new(
                GpuComponentFormat::Mat4,
                gpu_primitives_lod(),
                gpu_components::gpu_primitives_lod(),
            )),
            Box::new(lod::gpu_world_system()),
            Box::new(skinning::gpu_world_systems()),
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

    fn update(&self, _: &World) {}

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
    pub vertex_count: u32,
    pub instance_count: u32,
    pub base_index: u32,
    pub vertex_offset: i32,
    pub base_instance: u32,
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
