use std::{collections::HashMap, sync::Arc};

use ambient_core::transform::local_to_world;
use ambient_ecs::{query, ArchetypeFilter, EntityId, QueryState, World};
use ambient_gpu::{
    gpu::Gpu,
    mesh_buffer::{MeshBuffer, MeshMetadata},
    shader_module::{GraphicsPipeline, GraphicsPipelineInfo},
    typed_buffer::TypedBuffer,
};
use glam::{Mat4, UVec4, Vec3};
use itertools::Itertools;
use ordered_float::OrderedFloat;
use wgpu::BindGroup;

use super::{
    double_sided, get_gpu_primitive_id, primitives, FSMain, RendererResources, RendererShader, SharedMaterial, MATERIAL_BIND_GROUP,
    PRIMITIVES_BIND_GROUP,
};
use crate::{
    bind_groups::{self, BindGroups},
    transparency_group, RendererConfig,
};
use ambient_std::asset_cache::AssetCache;

pub struct TransparentRendererConfig {
    pub gpu: Arc<Gpu>,
    pub assets: AssetCache,
    pub renderer_config: RendererConfig,
    pub filter: ArchetypeFilter,
    pub targets: Vec<Option<wgpu::ColorTargetState>>,
    pub renderer_resources: RendererResources,
    pub fs_main: FSMain,
    pub render_opaque: bool,
}

pub struct TransparentRenderer {
    config: Arc<TransparentRendererConfig>,
    entity_primitive_count: HashMap<EntityId, usize>,
    primitives: Vec<TransparentPrimitive>,
    shaders: HashMap<String, Arc<ShaderNode>>,

    gpu_primitives: TypedBuffer<UVec4>,
    primitives_bind_group: wgpu::BindGroup,

    spawn_qs: QueryState,
    despawn_qs: QueryState,
}
impl TransparentRenderer {
    pub fn new(config: TransparentRendererConfig) -> Self {
        let gpu_primitives = TypedBuffer::new(
            config.gpu.clone(),
            "TransparentRenderer.primitives",
            1,
            1,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::INDIRECT,
        );

        Self {
            primitives: Vec::new(),
            shaders: HashMap::new(),
            entity_primitive_count: HashMap::new(),
            primitives_bind_group: Self::create_primitives_bind_group(
                &config.gpu,
                &config.renderer_resources.primitives_layout,
                gpu_primitives.buffer(),
            ),
            gpu_primitives,
            config: Arc::new(config),

            spawn_qs: QueryState::new(),
            despawn_qs: QueryState::new(),
        }
    }
    #[profiling::function]
    pub fn update(&mut self, world: &mut World, mesh_buffer: &MeshBuffer, camera_projection_view: Mat4) {
        let mut spawn_qs = std::mem::replace(&mut self.spawn_qs, QueryState::new());
        let mut despawn_qs = std::mem::replace(&mut self.despawn_qs, QueryState::new());
        for (id, (primitives,)) in query((primitives().changed(),)).filter(&self.config.filter).iter(world, Some(&mut spawn_qs)) {
            if let Some(primitive_count) = self.entity_primitive_count.get(&id) {
                for primitive_index in 0..*primitive_count {
                    self.remove(id, primitive_index);
                }
            }
            for (primitive_index, primitive) in primitives.iter().enumerate() {
                let primitive_shader = (primitive.shader)(&self.config.assets, &self.config.renderer_config);
                let transparent = primitive.material.transparent().unwrap_or(primitive_shader.transparent);
                if transparent || self.config.render_opaque {
                    let config = self.config.clone();
                    let double_sided =
                        world.get(id, double_sided()).unwrap_or(primitive.material.double_sided().unwrap_or(primitive_shader.double_sided));
                    let depth_write_enabled = primitive.material.depth_write_enabled().unwrap_or(primitive_shader.depth_write_enabled);
                    let shader = self
                        .shaders
                        .entry(primitive_shader.id.clone())
                        .or_insert_with(|| Arc::new(ShaderNode::new(config, primitive_shader.clone(), double_sided, depth_write_enabled)));
                    self.primitives.push(TransparentPrimitive {
                        id,
                        primitive_index,
                        shader: shader.clone(),
                        material: primitive.material.clone(),
                        mesh_metadata: MeshMetadata::default(),
                        transparency_group: world
                            .get(id, transparency_group())
                            .unwrap_or(primitive.material.transparency_group().unwrap_or(primitive_shader.transparency_group)),
                    });
                }
            }
            self.entity_primitive_count.insert(id, primitives.len());
        }
        for (id, _) in query(()).incl(primitives()).filter(&self.config.filter).despawned().iter(world, Some(&mut despawn_qs)) {
            if let Some(primitive_count) = self.entity_primitive_count.get(&id) {
                for primitive_index in 0..*primitive_count {
                    self.remove(id, primitive_index);
                }
            }
            self.entity_primitive_count.remove(&id);
        }
        self.spawn_qs = spawn_qs;
        self.despawn_qs = despawn_qs;
        for entry in self.primitives.iter_mut() {
            entry.material.update(world);
            let primitives = world.get_ref(entry.id, primitives()).unwrap();
            let mesh = &primitives[entry.primitive_index].mesh;
            entry.mesh_metadata = *mesh_buffer.get_mesh_metadata(mesh);
        }
        // TODO: Sort entities by distance to camera
        self.primitives.sort_by_key(|x| {
            let ltw = world.get(x.id, local_to_world()).unwrap();
            let transf = camera_projection_view * ltw;
            let point = transf.project_point3(Vec3::ZERO);
            (x.transparency_group, OrderedFloat(point.z))
        });

        if self.gpu_primitives.resize(self.primitives.len() as u64, true) {
            self.primitives_bind_group = Self::create_primitives_bind_group(
                &self.config.gpu,
                &self.config.renderer_resources.primitives_layout,
                self.gpu_primitives.buffer(),
            );
        }
        self.gpu_primitives
            .write(0, &self.primitives.iter().map(|e| get_gpu_primitive_id(world, e.id, e.primitive_index, 0)).collect_vec());
    }

    fn remove(&mut self, id: EntityId, primitive_index: usize) {
        self.primitives.retain(|x| !(x.id == id && x.primitive_index == primitive_index));
    }

    #[profiling::function]
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, bind_groups: &BindGroups<'a>) {
        let mut is_bound = false;
        // TODO: keep track of the state to avoid state switches (same pipeline multiple times etc.)
        for (i, entry) in self.primitives.iter().enumerate() {
            let bind_groups = [bind_groups.globals, bind_groups.entities, bind_groups.mesh_data, &self.primitives_bind_group];
            if !is_bound {
                for (i, bind_group) in bind_groups.iter().enumerate() {
                    render_pass.set_bind_group(i as _, bind_group, &[]);
                    is_bound = true
                }
            }
            let metadata = &entry.mesh_metadata;
            if metadata.index_count > 0 {
                render_pass.set_pipeline(entry.shader.pipeline.pipeline());
                render_pass.set_bind_group(bind_groups.len() as _, entry.material.bind_group(), &[]);
                // entry.shader.pipeline.bind(render_pass, MATERIAL_BIND_GROUP, entry.material.bind());

                render_pass.draw_indexed(
                    metadata.index_offset..(metadata.index_offset + metadata.index_count),
                    0,
                    (i as u32)..((i + 1) as u32),
                );
            }
        }
    }

    fn create_primitives_bind_group(gpu: &Gpu, layout: &wgpu::BindGroupLayout, buffer: &wgpu::Buffer) -> wgpu::BindGroup {
        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: buffer.as_entire_binding() }],
            label: Some("InstanceDataBuffer.bind_group"),
        })
    }
    pub fn n_entities(&self) -> usize {
        self.primitives.len()
    }
    pub fn dump(&self, f: &mut dyn std::io::Write) {
        writeln!(f, "    {} entities", self.primitives.len()).unwrap();
    }
}
struct TransparentPrimitive {
    id: EntityId,
    primitive_index: usize,
    shader: Arc<ShaderNode>,
    material: SharedMaterial,
    mesh_metadata: MeshMetadata,
    transparency_group: i32,
}
struct ShaderNode {
    pipeline: GraphicsPipeline,
}
impl ShaderNode {
    pub fn new(config: Arc<TransparentRendererConfig>, shader: Arc<RendererShader>, double_sided: bool, depth_write_enabled: bool) -> Self {
        let gpu = config.gpu.clone();

        let pipeline = shader.shader.to_pipeline(
            &gpu,
            GraphicsPipelineInfo {
                vs_main: &shader.vs_main,
                fs_main: shader.get_fs_main_name(config.fs_main),
                depth: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled,
                    depth_compare: wgpu::CompareFunction::Greater,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                targets: &config.targets,
                cull_mode: if double_sided { None } else { Some(wgpu::Face::Back) },
                ..Default::default()
            },
        );

        Self { pipeline }
    }
}
