use std::sync::Arc;

use ambient_core::{
    asset_cache,
    camera::*,
    gpu,
    gpu_ecs::{gpu_world, ENTITIES_BIND_GROUP},
    ui_scene,
};
use ambient_ecs::{ArchetypeFilter, Component, World};
use ambient_gpu::{
    gpu::{Gpu, GpuKey},
    mesh_buffer::MeshBuffer,
    shader_module::BindGroupDesc,
};
use ambient_std::{
    asset_cache::{AssetCache, SyncAssetKey, SyncAssetKeyExt},
    color::Color,
};
use glam::uvec2;
use wgpu::{BindGroupLayout, TextureView};

use super::{
    get_common_module, get_globals_module, get_resources_module,
    overlay_renderer::{OverlayConfig, OverlayRenderer},
    shadow_renderer::ShadowsRenderer,
    Culling, FSMain, ForwardGlobals, Outlines, OutlinesConfig, RenderTarget, RendererCollect, RendererCollectState, TransparentRenderer,
    TransparentRendererConfig, TreeRenderer, TreeRendererConfig,
};
use crate::{skinning::SkinsBufferKey, to_linear_format, ShaderDebugParams};
pub const GLOBALS_BIND_GROUP: &str = "GLOBALS_BIND_GROUP";
pub const MATERIAL_BIND_GROUP: &str = "MATERIAL_BIND_GROUP";
pub const RESOURCES_BIND_GROUP: &str = "RESOURCES_BIND_GROUP";
pub const PRIMITIVES_BIND_GROUP: &str = "PRIMITIVES_BIND_GROUP";
use ambient_gpu::mesh_buffer::MeshBufferKey;

pub const MESH_METADATA_BINDING: u32 = 0;
pub const MESH_POSITION_BINDING: u32 = 1;
pub const MESH_NORMAL_BINDING: u32 = 2;
pub const MESH_TANGENT_BINDING: u32 = 3;
pub const MESH_TEXCOORD0_BINDING: u32 = 4;
pub const MESH_JOINT_BINDING: u32 = 5;
pub const MESH_WEIGHT_BINDING: u32 = 6;
pub const SKINS_BINDING: u32 = 7;

#[derive(Clone)]
pub struct RendererResources {
    pub resources_layout: Arc<wgpu::BindGroupLayout>,
    pub globals_layout: Arc<wgpu::BindGroupLayout>,
    pub primitives_layout: Arc<wgpu::BindGroupLayout>,
    pub collect: Arc<RendererCollect>,
}

#[derive(Debug)]
struct RendererResourcesKey {
    pub shadow_cascades: u32,
}
impl SyncAssetKey<RendererResources> for RendererResourcesKey {
    fn load(&self, assets: AssetCache) -> RendererResources {
        let primitives = get_common_module(&assets).get_layout(PRIMITIVES_BIND_GROUP).unwrap().get(&assets);
        let resources_layout = get_resources_module().get_layout(RESOURCES_BIND_GROUP).unwrap().get(&assets);
        let globals_layout = get_globals_module(&assets, self.shadow_cascades).get_layout(GLOBALS_BIND_GROUP).unwrap().get(&assets);

        RendererResources {
            collect: Arc::new(RendererCollect::new(&assets)),
            primitives_layout: primitives,
            resources_layout,
            globals_layout,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RendererConfig {
    pub scene: Component<()>,
    pub shadows: bool,
    pub shadow_map_resolution: u32,
    pub shadow_cascades: u32,
    pub lod_cutoff_scaling: f32,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self { scene: ui_scene(), shadows: true, shadow_map_resolution: 1024, shadow_cascades: 5, lod_cutoff_scaling: 1. }
    }
}

pub enum RendererTarget<'a> {
    Target(&'a RenderTarget),
    Direct { color: &'a TextureView, depth: &'a TextureView, normals: &'a TextureView, size: wgpu::Extent3d },
}
impl<'a> RendererTarget<'a> {
    pub fn color(&self) -> &'a TextureView {
        match self {
            RendererTarget::Target(target) => &target.color_buffer_view,
            RendererTarget::Direct { color, .. } => color,
        }
    }
    pub fn depth(&self) -> &'a TextureView {
        match self {
            RendererTarget::Target(target) => &target.depth_buffer_view,
            RendererTarget::Direct { depth, .. } => depth,
        }
    }
    pub fn normals(&self) -> &'a TextureView {
        match self {
            RendererTarget::Target(target) => &target.normals_quat_buffer_view,
            RendererTarget::Direct { normals, .. } => normals,
        }
    }
    pub fn size(&self) -> wgpu::Extent3d {
        match self {
            RendererTarget::Target(target) => target.color_buffer.size,
            RendererTarget::Direct { size, .. } => *size,
        }
    }
}

pub trait SubRenderer: std::fmt::Debug + Send + Sync {
    fn render<'a>(
        &'a mut self,
        world: &World,
        mesh_buffer: &MeshBuffer,
        encoder: &mut wgpu::CommandEncoder,
        target: &RendererTarget,
        binds: &[(&str, &'a wgpu::BindGroup)],
    );
}

pub struct Renderer {
    gpu: Arc<Gpu>,
    pub config: RendererConfig,
    pub shader_debug_params: ShaderDebugParams,
    resources_layout: Arc<wgpu::BindGroupLayout>,
    culling: Culling,
    pub shadows: Option<ShadowsRenderer>,
    forward_globals: ForwardGlobals,
    forward_collect_state: RendererCollectState,
    forward: TreeRenderer,
    overlays: OverlayRenderer,
    transparent: TransparentRenderer,
    solids_frame: RenderTarget,
    outlines: Outlines,
    pub post_forward: Option<Box<dyn SubRenderer>>,
    pub post_transparent: Option<Box<dyn SubRenderer>>,
}
impl Renderer {
    pub fn new(_: &mut World, assets: AssetCache, config: RendererConfig) -> Self {
        let gpu = GpuKey.get(&assets);

        let renderer_resources = RendererResourcesKey { shadow_cascades: config.shadow_cascades }.get(&assets);

        // Need atleast one for array<Camera, SIZE> to be valid
        let shadow_cascades = config.shadow_cascades;

        let shadows =
            if config.shadows { Some(ShadowsRenderer::new(assets.clone(), renderer_resources.clone(), config.clone())) } else { None };

        let normals_format = to_linear_format(gpu.swapchain_format()).into();

        Self {
            culling: Culling::new(&assets, config.clone()),
            forward_globals: ForwardGlobals::new(gpu.clone(), renderer_resources.globals_layout.clone(), shadow_cascades, config.scene),
            forward_collect_state: RendererCollectState::new(&assets),
            shadows,
            overlays: OverlayRenderer::new(
                assets.clone(),
                config.clone(),
                OverlayConfig {
                    fs_main: FSMain::Forward,
                    gpu: gpu.clone(),
                    targets: vec![Some(gpu.swapchain_format().into())],
                    resources: renderer_resources.clone(),
                },
            ),
            forward: TreeRenderer::new(TreeRendererConfig {
                gpu: gpu.clone(),
                assets: assets.clone(),
                renderer_config: config.clone(),
                targets: vec![Some(gpu.swapchain_format().into()), Some(normals_format)],
                filter: ArchetypeFilter::new().incl(config.scene),
                renderer_resources: renderer_resources.clone(),
                fs_main: FSMain::Forward,
                opaque_only: true,
                depth_stencil: true,
                cull_mode: Some(wgpu::Face::Back),
                depth_bias: Default::default(),
            }),
            transparent: TransparentRenderer::new(TransparentRendererConfig {
                gpu: gpu.clone(),
                assets: assets.clone(),
                renderer_config: config.clone(),
                targets: vec![Some(wgpu::ColorTargetState {
                    format: gpu.swapchain_format(),
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                filter: ArchetypeFilter::new().incl(config.scene),
                renderer_resources: renderer_resources.clone(),
                fs_main: FSMain::Forward,
                render_opaque: false,
            }),
            solids_frame: RenderTarget::new(
                gpu.clone(),
                uvec2(1, 1),
                Some(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST),
            ),
            outlines: Outlines::new(
                &assets,
                OutlinesConfig { scene: config.scene, renderer_resources: renderer_resources.clone() },
                config.clone(),
            ),
            resources_layout: renderer_resources.resources_layout,
            config,
            shader_debug_params: Default::default(),
            gpu,
            post_forward: Default::default(),
            post_transparent: Default::default(),
        }
    }

    pub fn render(
        &mut self,
        world: &mut World,
        encoder: &mut wgpu::CommandEncoder,
        post_submit: &mut Vec<Box<dyn FnOnce() + Send + Send>>,
        target: RendererTarget,
        clear: Option<Color>,
    ) {
        profiling::scope!("Renderer.render");

        if let RendererTarget::Target(target) = &target {
            if self.solids_frame.color_buffer.size != target.color_buffer.size {
                self.solids_frame = RenderTarget::new(
                    self.gpu.clone(),
                    uvec2(target.color_buffer.size.width, target.color_buffer.size.height),
                    Some(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST),
                );
            }
        }

        let mesh_buffer_h = MeshBufferKey.get(world.resource(asset_cache()));
        let mesh_buffer = mesh_buffer_h.lock();

        let resources_bind_group = create_resources_bind_group(world, &self.resources_layout, &mesh_buffer);
        let entities_bind_group = {
            let gpu_world_h = world.resource(gpu_world()).clone();
            let gpu_world = gpu_world_h.lock();
            gpu_world.create_bind_group(true)
        };

        let main_camera = Camera::get_active(world, self.config.scene).unwrap_or_default();
        {
            profiling::scope!("Update");
            self.culling.run(encoder, world, &Default::default());

            self.forward_collect_state.set_camera(0);
            self.forward.update(world);
            self.overlays.update(world);
            self.forward.run_collect(encoder, post_submit, &resources_bind_group, &entities_bind_group, &mut self.forward_collect_state);
            self.transparent.update(world, &mesh_buffer, main_camera.projection_view());
        }

        if let Some(shadows) = &mut self.shadows {
            shadows.run(world, encoder, post_submit, &resources_bind_group, &entities_bind_group, &mesh_buffer);
        }

        self.forward_globals.params.debug_params = self.shader_debug_params;
        self.forward_globals.update(world, &self.shadows.as_ref().map(|x| x.get_cameras()).unwrap_or_default());
        let forward_globals_bind_group = self.forward_globals.create_bind_group(
            world.resource(asset_cache()).clone(),
            self.shadows.as_ref().map(|x| &x.shadow_view),
            &self.solids_frame,
        );

        let binds = [
            (RESOURCES_BIND_GROUP, &resources_bind_group),
            (ENTITIES_BIND_GROUP, &entities_bind_group),
            (GLOBALS_BIND_GROUP, &forward_globals_bind_group),
        ];

        {
            profiling::scope!("Forward");
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Forward"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: target.color(),
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: if let Some(clear) = clear { wgpu::LoadOp::Clear(clear.into()) } else { wgpu::LoadOp::Load },
                            store: true,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: target.normals(),
                        resolve_target: None,
                        ops: wgpu::Operations {
                            /// clear color is ignored as the normal buffer should always be initialized with black
                            load: if clear.is_some() { wgpu::LoadOp::Clear(Color::BLACK.into()) } else { wgpu::LoadOp::Load },
                            store: true,
                        },
                    }),
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: target.depth(),
                    depth_ops: Some(wgpu::Operations {
                        load: if clear.is_some() { wgpu::LoadOp::Clear(0.0) } else { wgpu::LoadOp::Load },
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            render_pass.set_index_buffer(mesh_buffer.index_buffer.buffer().slice(..), wgpu::IndexFormat::Uint32);

            self.forward.render(&mut render_pass, &self.forward_collect_state, &binds);
            {
                profiling::scope!("Drop render pass");
                drop(render_pass);
            }
        }

        if let Some(post_forward) = &mut self.post_forward {
            post_forward.render(
                world,
                &mesh_buffer,
                encoder,
                &target,
                &[(RESOURCES_BIND_GROUP, &resources_bind_group), (GLOBALS_BIND_GROUP, &forward_globals_bind_group)],
            );
        }

        {
            let binds = [(GLOBALS_BIND_GROUP, &forward_globals_bind_group)];

            self.overlays.render(encoder, &target, &binds, &mesh_buffer);
        }

        if let RendererTarget::Target(target) = &target {
            encoder.copy_texture_to_texture(
                target.depth_buffer.handle.as_image_copy(),
                self.solids_frame.depth_buffer.handle.as_image_copy(),
                target.depth_buffer.size,
            );
            encoder.copy_texture_to_texture(
                target.color_buffer.handle.as_image_copy(),
                self.solids_frame.color_buffer.handle.as_image_copy(),
                target.color_buffer.size,
            );
            encoder.copy_texture_to_texture(
                target.normals_quat_buffer.handle.as_image_copy(),
                self.solids_frame.normals_quat_buffer.handle.as_image_copy(),
                target.normals_quat_buffer.size,
            );
        }
        {
            profiling::scope!("Transparent");
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Transparent"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target.color(),
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: true },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: target.depth(),
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Load, store: true }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_index_buffer(mesh_buffer.index_buffer.buffer().slice(..), wgpu::IndexFormat::Uint32);

            self.transparent.render(&mut render_pass, &binds);

            {
                profiling::scope!("Drop render pass");
                drop(render_pass);
            }
        }

        if let Some(post_transparent) = &mut self.post_transparent {
            post_transparent.render(
                world,
                &mesh_buffer,
                encoder,
                &target,
                &[(RESOURCES_BIND_GROUP, &resources_bind_group), (GLOBALS_BIND_GROUP, &forward_globals_bind_group)],
            );
        }

        self.outlines.render(world, encoder, post_submit, &target, &binds, &mesh_buffer);
    }

    pub fn dump_to_tmp_file(&self) {
        std::fs::create_dir_all("tmp").expect("Failed to create tmp dir");
        let mut f = std::fs::File::create("tmp/renderer.txt").expect("Unable to create file");
        self.dump(&mut f);
        log::info!("Wrote renderer to tmp/renderer.txt");
    }

    pub fn is_rendered(&self) -> bool {
        #[cfg(target_os = "macos")]
        let res = self.forward_collect_state.counts_cpu.lock().len() == self.forward_collect_state.counts.len() as usize;
        #[cfg(not(target_os = "macos"))]
        let res = true;
        res
    }
    pub fn n_entities(&self) -> usize {
        self.forward.n_entities()
    }
    pub fn stats(&self) -> String {
        format!(
            "{} forward: {}/{} transparent: {}",
            self.shadows.as_ref().map(|x| x.stats()).unwrap_or_default(),
            self.forward.n_entities(),
            self.forward.n_nodes(),
            self.transparent.n_entities()
        )
    }
    pub fn dump(&self, f: &mut dyn std::io::Write) {
        if let Some(shadows) = &self.shadows {
            shadows.dump(f);
        }
        writeln!(f, "  forward").unwrap();
        self.forward.dump(f);
        writeln!(f, "  transparent").unwrap();
        self.transparent.dump(f);
        writeln!(f, "  outlines").unwrap();
        self.outlines.dump(f);
    }
}

impl std::fmt::Debug for Renderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Renderer").finish()
    }
}

pub(crate) fn get_resources_layout() -> BindGroupDesc {
    fn resource_storage_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    BindGroupDesc {
        entries: vec![
            resource_storage_entry(MESH_METADATA_BINDING),
            resource_storage_entry(MESH_POSITION_BINDING),
            resource_storage_entry(MESH_NORMAL_BINDING),
            resource_storage_entry(MESH_TANGENT_BINDING),
            resource_storage_entry(MESH_TEXCOORD0_BINDING),
            resource_storage_entry(MESH_JOINT_BINDING),
            resource_storage_entry(MESH_WEIGHT_BINDING),
            resource_storage_entry(SKINS_BINDING),
        ],
        label: RESOURCES_BIND_GROUP.into(),
    }
}

fn create_resources_bind_group(world: &World, layout: &BindGroupLayout, mesh_buffer: &MeshBuffer) -> wgpu::BindGroup {
    let gpu = world.resource(gpu()).clone();
    let skins_h = SkinsBufferKey.get(world.resource(asset_cache()));
    let skins = skins_h.lock();
    gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout,
        entries: &[
            wgpu::BindGroupEntry { binding: MESH_METADATA_BINDING, resource: mesh_buffer.metadata_buffer.buffer().as_entire_binding() },
            wgpu::BindGroupEntry { binding: MESH_POSITION_BINDING, resource: mesh_buffer.position_buffer.buffer().as_entire_binding() },
            wgpu::BindGroupEntry { binding: MESH_NORMAL_BINDING, resource: mesh_buffer.normal_buffer.buffer().as_entire_binding() },
            wgpu::BindGroupEntry { binding: MESH_TANGENT_BINDING, resource: mesh_buffer.tangent_buffer.buffer().as_entire_binding() },
            wgpu::BindGroupEntry { binding: MESH_TEXCOORD0_BINDING, resource: mesh_buffer.texcoord0_buffer.buffer().as_entire_binding() },
            wgpu::BindGroupEntry { binding: MESH_JOINT_BINDING, resource: mesh_buffer.joint_buffer.buffer().as_entire_binding() },
            wgpu::BindGroupEntry { binding: MESH_WEIGHT_BINDING, resource: mesh_buffer.weight_buffer.buffer().as_entire_binding() },
            wgpu::BindGroupEntry { binding: SKINS_BINDING, resource: skins.buffer.buffer().as_entire_binding() },
        ],
        label: Some("resources_bind_group"),
    })
}
