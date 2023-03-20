use std::{fmt::Debug, sync::Arc};

use ambient_core::{asset_cache, camera::Camera, main_scene, player::local_user_id};
use ambient_ecs::World;
use ambient_gpu::{
    gpu::{Gpu, GpuKey},
    mesh_buffer::{GpuMesh, MeshBuffer},
    shader_module::{BindGroupDesc, GraphicsPipeline, GraphicsPipelineInfo, Shader, ShaderModule},
    typed_buffer::TypedBuffer,
};
use ambient_meshes::QuadMeshKey;
use ambient_renderer::{
    bind_groups::BindGroups, get_mesh_data_module, get_overlay_modules, PostSubmitFunc, RendererTarget, SubRenderer, GLOBALS_BIND_GROUP,
    GLOBALS_BIND_GROUP_SIZE,
};
use ambient_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    include_file,
};
use bytemuck::{Pod, Zeroable};
use glam::{vec2, Mat4, Quat, Vec2, Vec3};
use once_cell::sync::OnceCell;
use wgpu::{BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry, BlendState, BufferUsages, ColorTargetState, ColorWrites, ShaderStages};

use super::{gizmos, GizmoPrimitive};

fn get_gizmos_layout() -> BindGroupDesc<'static> {
    BindGroupDesc {
        entries: vec![
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Depth,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
        ],
        label: "GIZMOS_BIND_GROUP".into(),
    }
}

pub struct GizmoRenderer {
    gpu: Arc<Gpu>,
    quad: Arc<GpuMesh>,
    pipeline: OnceCell<GraphicsPipeline>,
    buffer: TypedBuffer<Gizmo>,
    primitives: Vec<Gizmo>,
    layout: Arc<BindGroupLayout>,
}
impl Debug for GizmoRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GizmoRenderer").finish()
    }
}

impl GizmoRenderer {
    pub fn new(assets: &AssetCache) -> Self {
        let gpu = GpuKey.get(assets);
        let buffer =
            TypedBuffer::new(gpu.clone(), "Gizmo Buffer", 128, 0, BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC);

        let layout = get_gizmos_layout().get(assets);

        Self { gpu, quad: QuadMeshKey.get(assets), pipeline: OnceCell::new(), buffer, primitives: Vec::new(), layout }
    }
}

impl SubRenderer for GizmoRenderer {
    #[profiling::function]
    fn render<'a>(
        &'a mut self,
        world: &World,
        mesh_buffer: &MeshBuffer,
        encoder: &mut wgpu::CommandEncoder,
        target: &RendererTarget,
        bind_groups: &BindGroups<'a>,
        _: &mut Vec<PostSubmitFunc>,
    ) {
        let gizmos = world.resource(gizmos());
        let camera = Camera::get_active(world, main_scene(), world.resource_opt(local_user_id())).unwrap_or_default();
        let primitives = &mut self.primitives;

        primitives.clear();

        gizmos.scopes().for_each(|scope| {
            primitives.extend(scope.primitives.iter().map(|v| Gizmo::from_primitive(v, camera.position())));
        });

        if primitives.is_empty() {
            // return;
        }

        let assets = world.resource(asset_cache());
        let gpu = &self.gpu;
        let pipeline = self.pipeline.get_or_init(|| {
            let layout = get_gizmos_layout();

            let source = include_file!("gizmos.wgsl");
            let shader = Shader::new(
                assets,
                "gizmos",
                &[GLOBALS_BIND_GROUP, "GIZMOS_BIND_GROUP"],
                &ShaderModule::new("Gizmo", source)
                    .with_binding_desc(layout)
                    .with_dependencies(get_overlay_modules(assets, 1))
                    .with_dependency(get_mesh_data_module(GLOBALS_BIND_GROUP_SIZE)),
            )
            .unwrap();

            shader.to_pipeline(
                gpu,
                GraphicsPipelineInfo {
                    targets: &[Some(ColorTargetState {
                        format: gpu.swapchain_format(),
                        blend: Some(BlendState::ALPHA_BLENDING),
                        write_mask: ColorWrites::ALL,
                    })],
                    cull_mode: None,
                    ..Default::default()
                },
            )
        });

        self.buffer.fill(primitives, |_| {
            log::debug!("Resizing bind group for gizmos");
        });

        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Gizmo bind group"),
            layout: &self.layout,
            entries: &[
                BindGroupEntry { binding: 0, resource: wgpu::BindingResource::Buffer(self.buffer.buffer().as_entire_buffer_binding()) },
                BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(target.depth()) },
            ],
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Gizmos"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target.color(),
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: true },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_index_buffer(mesh_buffer.index_buffer.buffer().slice(..), wgpu::IndexFormat::Uint32);

        let indices = mesh_buffer.indices_of(&self.quad);
        render_pass.set_pipeline(pipeline.pipeline());

        let bind_groups = [bind_groups.globals, &bind_group];
        for (index, bind_group) in bind_groups.iter().enumerate() {
            render_pass.set_bind_group(index as _, bind_group, &[])
        }

        render_pass.draw_indexed(indices, 0, 0..primitives.len() as _);
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C)]
struct Gizmo {
    model: Mat4,
    color: Vec3,
    corner: f32,
    scale: Vec2,
    border_width: f32,
    inner_corner: f32,
}

impl Gizmo {
    pub fn from_primitive(prim: &GizmoPrimitive, camera_pos: Vec3) -> Self {
        match *prim {
            GizmoPrimitive::Sphere { origin, radius, color, border_width } => Self {
                model: Mat4::from_scale_rotation_translation(
                    Vec3::splat(radius),
                    Quat::from_rotation_arc(Vec3::Z, (origin - camera_pos).normalize()),
                    origin,
                ),
                color,
                corner: 1.,
                border_width,
                scale: Vec2::splat(radius),
                inner_corner: 1.,
            },
            GizmoPrimitive::Line { start, end, radius, color } => {
                let dir = start - end;
                let len = dir.length();
                let dir = dir.normalize_or_zero();
                let mid = (start + end) / 2.;
                let billboard = (mid - camera_pos).reject_from(dir).normalize_or_zero();

                let rot = Quat::from_rotation_arc(Vec3::X, dir);
                let tan = rot * Vec3::Z;
                assert!(tan.is_normalized());

                let scale = vec2(len * 0.5, radius);
                Self {
                    model: Mat4::from_scale_rotation_translation(scale.extend(0.), Quat::from_rotation_arc(tan, billboard) * rot, mid),
                    color,
                    corner: 1.,
                    border_width: len,
                    scale,
                    inner_corner: 0.0,
                }
            }
            GizmoPrimitive::Rect { origin, extents, corner: corner_radius, inner_corner, thickness, normal, color } => Self {
                model: Mat4::from_scale_rotation_translation(extents.extend(0.), Quat::from_rotation_arc(Vec3::Z, normal), origin),
                color,
                corner: corner_radius,
                scale: extents,
                border_width: thickness,
                inner_corner,
            },
        }
    }
}
