use std::{collections::HashMap, sync::Arc};

use elements_core::transform::translation;
use elements_ecs::{query, EntityId, QueryState, World};
use elements_gpu::{
    gpu::Gpu, mesh_buffer::{GpuMesh, MeshBuffer}, shader_module::{GraphicsPipeline, GraphicsPipelineInfo}
};
use elements_meshes::QuadMeshKey;
use elements_std::asset_cache::{AssetCache, SyncAssetKeyExt};
use ordered_float::OrderedFloat;
use wgpu::{
    BindGroup, ColorTargetState, CommandEncoder, IndexFormat, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipeline
};

use super::{
    material, overlay, renderer_shader, FSMain, RendererResources, RendererShader, RendererTarget, SharedMaterial, MATERIAL_BIND_GROUP
};

struct OverlayEntity {
    id: EntityId,
    depth: OrderedFloat<f32>,
    material: SharedMaterial,
    shader: usize,
}

pub struct OverlayConfig {
    pub gpu: Arc<Gpu>,
    pub fs_main: FSMain,
    pub targets: Vec<Option<ColorTargetState>>,
    pub resources: RendererResources,
}

pub struct OverlayRenderer {
    config: OverlayConfig,
    pipelines_map: HashMap<String, usize>,
    pipelines: Vec<GraphicsPipeline>,
    mesh: Arc<GpuMesh>,
    entities: Vec<OverlayEntity>,
    spawn_qs: QueryState,
    despawn_qs: QueryState,
}

impl OverlayRenderer {
    pub fn new(assets: AssetCache, config: OverlayConfig) -> Self {
        Self {
            config,
            entities: Vec::new(),
            spawn_qs: QueryState::new(),
            despawn_qs: QueryState::new(),
            mesh: QuadMeshKey.get(&assets),
            pipelines: Default::default(),
            pipelines_map: Default::default(),
        }
    }

    pub fn update(&mut self, world: &mut World) {
        let mut spawn_qs = std::mem::replace(&mut self.spawn_qs, QueryState::new());
        let mut despawn_qs = std::mem::replace(&mut self.despawn_qs, QueryState::new());
        for (id, ((), shader, material, pos)) in
            query((overlay(), renderer_shader().changed(), material().changed(), translation())).iter(world, Some(&mut spawn_qs))
        {
            self.remove(id);

            let shader = self.shader(shader).0;

            // Insert again
            self.entities.push(OverlayEntity { shader, id, depth: OrderedFloat(pos.z), material: material.clone() })
        }

        let removed = query((overlay(),))
            .despawned()
            .iter(world, Some(&mut despawn_qs))
            .map(|(id, ((),))| self.remove(id).expect("Entity not in renderer"))
            .count();

        if removed > 0 {
            self.entities.sort_by_key(|v| v.depth)
        };
        self.spawn_qs = spawn_qs;
        self.despawn_qs = despawn_qs;
    }

    fn remove(&mut self, id: EntityId) -> Option<()> {
        if let Some(id) = self.entities.iter().position(|v| v.id == id) {
            self.entities.remove(id);
            Some(())
        } else {
            None
        }
    }

    fn shader(&mut self, shader: &RendererShader) -> (usize, &RenderPipeline) {
        let config = &self.config;
        match self.pipelines_map.entry(shader.id.to_owned()) {
            std::collections::hash_map::Entry::Occupied(entry) => {
                let idx = *entry.get();
                (idx, self.pipelines[idx].pipeline())
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                // Create the effect
                let idx = self.pipelines.len();

                let pipeline = shader.shader.to_pipeline(
                    &config.gpu,
                    GraphicsPipelineInfo {
                        vs_main: &shader.vs_main,
                        fs_main: shader.get_fs_main_name(config.fs_main),
                        targets: &config.targets,
                        ..Default::default()
                    }
                    .with_depth(),
                );

                self.pipelines.push(pipeline);
                entry.insert(idx);
                (idx, self.pipelines[idx].pipeline())
            }
        }
    }

    pub fn render<'a>(
        &'a self,
        cmds: &mut CommandEncoder,
        target: &RendererTarget,
        binds: &[(&str, &BindGroup)],
        mesh_buffer: &MeshBuffer,
    ) {
        let mut renderpass = cmds.begin_render_pass(&RenderPassDescriptor {
            label: Some("Overlay"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: target.color(),
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: true },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: target.depth(),
                depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Load, store: true }),
                stencil_ops: None,
            }),
        });

        renderpass.set_index_buffer(mesh_buffer.index_buffer.buffer().slice(..), IndexFormat::Uint32);

        let mut is_bound = false;

        for e in &self.entities {
            let indices = mesh_buffer.indices_of(&self.mesh);

            let pipeline = &self.pipelines[e.shader];
            if !is_bound {
                for (name, group) in binds {
                    pipeline.bind(&mut renderpass, name, group);
                    is_bound = true
                }
            }
            renderpass.set_pipeline(pipeline.pipeline());
            let material = &e.material;
            pipeline.bind(&mut renderpass, MATERIAL_BIND_GROUP, material.bind());

            renderpass.draw_indexed(indices, 0, 0..1);
        }
    }
}
