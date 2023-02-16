use std::sync::Arc;

use glam::{Mat4, UVec3, Vec3, Vec4};
use kiwi_core::{
    asset_cache, mesh,
    transform::{mesh_to_local, scale},
    ui_scene,
};
use kiwi_ecs::{components, query, EntityData, SystemGroup, World};
use kiwi_element::{element_component, Element, ElementComponentExt, Hooks};
use kiwi_gpu::{
    gpu::GpuKey,
    shader_module::{BindGroupDesc, ShaderModule},
    typed_buffer::TypedBuffer,
};
use kiwi_meshes::UIRectMeshKey;
use kiwi_renderer::{
    gpu_primitives, material, primitives, renderer_shader, Material, MaterialShader, RendererConfig, RendererShader, SharedMaterial,
    StandardShaderKey, MATERIAL_BIND_GROUP,
};
use kiwi_std::{
    asset_cache::{AssetCache, SyncAssetKey, SyncAssetKeyExt},
    cb,
    color::Color,
    friendly_id, include_file,
};
use wgpu::BindGroup;

use crate::{gpu_ui_size, mesh_to_local_from_size, UIBase};

components!("ui", {
    background_color: Color,
    border_color: Color,
    border_radius: Corners,
    border_thickness: f32,
    ui_rect: (),
});

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
        Self { top_left: value, top_right: value, bottom_left: value, bottom_right: value }
    }
}

/// A simple UI rect. Use components such as `width`, `height`, `background_color`, `border_color`, `border_radius` and `border_thickness`
/// to control its appearance
#[element_component]
pub fn Rectangle(_hooks: &mut Hooks) -> Element {
    with_rect(UIBase.el())
}

pub(crate) fn with_rect(element: Element) -> Element {
    element
        .init(ui_rect(), ())
        .init(gpu_ui_size(), Vec4::ZERO)
        .init(mesh_to_local(), Mat4::IDENTITY)
        .init(primitives(), vec![])
        .init_default(gpu_primitives())
        .init(scale(), Vec3::ONE)
        .init(mesh_to_local_from_size(), ())
        .init(ui_scene(), ())
}

pub fn systems() -> SystemGroup {
    SystemGroup::new(
        "ui/rect",
        vec![
            query(()).incl(ui_rect()).excl(mesh()).to_system(|q, world, qs, _| {
                let assets = world.resource(asset_cache()).clone();
                for (id, _) in q.collect_cloned(world, qs) {
                    world
                        .add_components(
                            id,
                            EntityData::new().set(mesh(), UIRectMeshKey.get(&assets)).set(renderer_shader(), cb(get_rect_shader)),
                        )
                        .unwrap();
                }
            }),
            query(())
                .incl(ui_rect())
                .optional_changed(background_color())
                .optional_changed(border_color())
                .optional_changed(border_radius())
                .optional_changed(border_thickness())
                .to_system(|q, world, qs, _| {
                    let assets = world.resource(asset_cache()).clone();
                    for (id, _) in q.collect_cloned(world, qs) {
                        world
                            .add_component(
                                id,
                                material(),
                                RectMaterialKey {
                                    params: RectMaterialParams {
                                        background_color: world.get(id, background_color()).unwrap_or(Color::WHITE).into(),
                                        border_color: world.get(id, border_color()).unwrap_or(Color::WHITE).into(),
                                        border_radius: world.get(id, border_radius()).unwrap_or_default(),
                                        border_thickness: world.get(id, border_thickness()).unwrap_or(0.),
                                        _padding: Default::default(),
                                    },
                                }
                                .get(&assets),
                            )
                            .unwrap();
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
            shader: ShaderModule::new(
                "RectMaterial",
                include_file!("rect.wgsl"),
                vec![BindGroupDesc {
                    entries: vec![wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: MATERIAL_BIND_GROUP.into(),
                }
                .into()],
            ),

            id: "rect_material_shader".to_string(),
        })
    }
}

pub fn get_rect_shader(assets: &AssetCache, config: &RendererConfig) -> Arc<RendererShader> {
    StandardShaderKey { material_shader: RectMaterialShaderKey.get(assets), lit: false, shadow_cascades: config.shadow_cascades }
        .get(assets)
}

#[derive(Debug)]
pub struct RectMaterialKey {
    pub params: RectMaterialParams,
}
impl SyncAssetKey<SharedMaterial> for RectMaterialKey {
    fn load(&self, assets: AssetCache) -> SharedMaterial {
        SharedMaterial::new(RectMaterial::new(assets, self.params))
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
}
impl RectMaterial {
    pub fn new(assets: AssetCache, params: RectMaterialParams) -> Self {
        let gpu = GpuKey.get(&assets);
        let layout = RectMaterialShaderKey.get(&assets).shader.first_layout(&assets);
        let buffer = TypedBuffer::new_init(
            gpu.clone(),
            "RectMaterial.buffer",
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            &[params],
        );
        Self {
            id: friendly_id(),
            bind_group: gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(buffer.buffer().as_entire_buffer_binding()),
                }],
                label: Some("RectMaterial.bind_group"),
            }),
            transparent: Some(params.background_color.w != 0. || params.border_color.w != 0.),
        }
    }
}
impl std::fmt::Debug for RectMaterial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RectMaterial").field("id", &self.id).finish()
    }
}
impl Material for RectMaterial {
    fn bind(&self) -> &BindGroup {
        &self.bind_group
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn transparent(&self) -> Option<bool> {
        self.transparent
    }
}
