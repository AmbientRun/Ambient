use std::sync::Arc;

use ambient_core::{asset_cache, mesh};
use ambient_ecs::{
    components, ensure_has_component_with_default, query, Debuggable, Description, Entity, Name, Networked, Store, SystemGroup,
};
use ambient_gpu::{
    gpu::GpuKey,
    shader_module::{BindGroupDesc, ShaderModule},
    typed_buffer::TypedBuffer,
};
use ambient_meshes::UIRectMeshKey;
use ambient_renderer::{
    gpu_primitives, material, primitives, renderer_shader, Material, MaterialShader, RendererConfig, RendererShader, SharedMaterial,
    StandardShaderKey, MATERIAL_BIND_GROUP,
};
use ambient_std::{
    asset_cache::{AssetCache, SyncAssetKey, SyncAssetKeyExt},
    cb,
    color::Color,
    friendly_id, include_file,
};
use glam::{vec4, UVec3, Vec4};
use wgpu::BindGroup;

components!("ui", {
    @[Debuggable, Networked, Store, Name["Background color"], Description["Background color of an entity with a `rect` component."]]
    background_color: Vec4,
    @[Debuggable, Networked, Store, Name["Border color"], Description["Border color of an entity with a `rect` component."]]
    border_color: Vec4,
    @[Debuggable, Networked, Store, Name["Border radius"], Description["Radius for each corner of an entity with a `rect` component.\n`x` = top-left, `y` = top-right, `z` = bottom-left, `w` = bottom-right."]]
    border_radius: Vec4,
    @[Debuggable, Networked, Store, Name["Border thickness"], Description["Border thickness of an entity with a `rect` component."]]
    border_thickness: f32,
    @[Debuggable, Networked, Store, Name["Rect"], Description["If attached to an entity, the entity will be converted to a UI rectangle, with optionally rounded corners and borders."]]
    rect: (),
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
impl From<Corners> for Vec4 {
    fn from(value: Corners) -> Self {
        vec4(value.top_left, value.top_right, value.bottom_left, value.bottom_right)
    }
}
impl From<Vec4> for Corners {
    fn from(value: Vec4) -> Self {
        Self { top_left: value.x, top_right: value.y, bottom_left: value.z, bottom_right: value.w }
    }
}

pub fn systems() -> SystemGroup {
    SystemGroup::new(
        "ui/rect",
        vec![
            ensure_has_component_with_default(rect(), primitives()),
            ensure_has_component_with_default(rect(), gpu_primitives()),
            query(()).incl(rect()).excl(mesh()).to_system(|q, world, qs, _| {
                let assets = world.resource(asset_cache()).clone();
                for (id, _) in q.collect_cloned(world, qs) {
                    world
                        .add_components(
                            id,
                            Entity::new().with(mesh(), UIRectMeshKey.get(&assets)).with(renderer_shader(), cb(get_rect_shader)),
                        )
                        .unwrap();
                }
            }),
            query(())
                .incl(rect())
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
                                        background_color: world.get(id, background_color()).unwrap_or(Color::WHITE.into()),
                                        border_color: world.get(id, border_color()).unwrap_or(Color::WHITE.into()),
                                        border_radius: world.get(id, border_radius()).unwrap_or_default().into(),
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
