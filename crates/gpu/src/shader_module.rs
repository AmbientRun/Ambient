use std::{collections::HashMap, sync::Arc};

use elements_std::{asset_cache::*, CowStr};
use itertools::Itertools;
use wgpu::{ComputePipelineDescriptor, DepthBiasState};

use super::gpu::{Gpu, GpuKey, DEFAULT_SAMPLE_COUNT};

#[derive(Debug, Clone, PartialEq)]
pub enum WgslValue {
    String(CowStr),
    Raw(CowStr),
    Float(f32),
    Int32(u32),
    Int64(u64),
}

impl WgslValue {
    pub fn as_integer(&self) -> Option<u32> {
        match self {
            WgslValue::Int32(v) => Some(*v),
            _ => None,
        }
    }

    fn to_wgsl(&self) -> String {
        match self {
            WgslValue::String(v) => format!("{v:?}"),
            WgslValue::Raw(v) => v.to_string(),
            WgslValue::Float(v) => v.to_string(),
            WgslValue::Int32(v) => v.to_string(),
            WgslValue::Int64(v) => v.to_string(),
        }
    }
}

impl From<&'static str> for WgslValue {
    fn from(v: &'static str) -> Self {
        Self::String(v.into())
    }
}
impl From<String> for WgslValue {
    fn from(v: String) -> Self {
        Self::String(v.into())
    }
}

impl From<f32> for WgslValue {
    fn from(v: f32) -> Self {
        Self::Float(v)
    }
}

impl From<u32> for WgslValue {
    fn from(v: u32) -> Self {
        Self::Int32(v)
    }
}

impl From<u64> for WgslValue {
    fn from(v: u64) -> Self {
        Self::Int64(v)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShaderModuleIdentifier {
    BindGroup(BindGroupDesc),
    Constant { name: CowStr, value: WgslValue },
}

impl ShaderModuleIdentifier {
    pub fn bind_group(desc: BindGroupDesc) -> Self {
        Self::BindGroup(desc)
    }

    pub fn raw(name: impl Into<CowStr>, value: impl Into<CowStr>) -> Self {
        Self::Constant { name: name.into(), value: WgslValue::Raw(value.into()) }
    }

    pub fn constant(name: impl Into<CowStr>, value: impl Into<WgslValue>) -> Self {
        Self::Constant { name: name.into(), value: value.into() }
    }

    pub fn name(&self) -> &str {
        match self {
            ShaderModuleIdentifier::BindGroup(v) => &v.label,
            ShaderModuleIdentifier::Constant { name, .. } => name,
        }
    }

    pub fn as_bind_group(&self) -> Option<&BindGroupDesc> {
        match self {
            ShaderModuleIdentifier::BindGroup(v) => Some(v),
            _ => None,
        }
    }
}

impl From<BindGroupDesc> for ShaderModuleIdentifier {
    fn from(desc: BindGroupDesc) -> Self {
        Self::bind_group(desc)
    }
}

/// Defines a part of a shader and it's preprocessed bind groups
#[derive(Clone, Debug, Default)]
pub struct ShaderModule {
    pub label: CowStr,
    pub source: CowStr,
    // Use the label to preprocess constants
    pub idents: Vec<ShaderModuleIdentifier>,
}

impl ShaderModule {
    pub fn new(label: impl Into<CowStr>, source: impl Into<CowStr>, idents: Vec<ShaderModuleIdentifier>) -> Self {
        Self { label: label.into(), source: source.into(), idents }
    }

    /// With empty idents
    pub fn from_str(label: impl Into<CowStr>, source: impl Into<CowStr>) -> ShaderModule {
        Self { label: label.into(), source: source.into(), ..Default::default() }
    }

    pub fn get_layout(&self, name: &str) -> Option<&BindGroupDesc> {
        self.get(name).and_then(ShaderModuleIdentifier::as_bind_group)
    }

    pub fn first_layout(&self, assets: &AssetCache) -> Arc<wgpu::BindGroupLayout> {
        self.idents
            .iter()
            .find_map(|v| match v {
                ShaderModuleIdentifier::BindGroup(v) => Some(v.get(assets)),
                _ => None,
            })
            .unwrap()
    }

    pub(crate) fn get(&self, name: &str) -> Option<&ShaderModuleIdentifier> {
        self.idents.iter().find(|v| match v {
            ShaderModuleIdentifier::BindGroup(v) => v.label == name,
            ShaderModuleIdentifier::Constant { name: n, .. } => n == name,
        })
    }
}

impl<'a> FromIterator<&'a ShaderModule> for ShaderModule {
    fn from_iter<T: IntoIterator<Item = &'a Self>>(iter: T) -> Self {
        let mut source = String::new();
        let mut idents = Vec::new();
        let mut label = String::new();
        for v in iter {
            source.push_str(&v.source[..]);
            idents.extend_from_slice(&v.idents);
            if !label.is_empty() {
                label.push(':');
            }
            label.push_str(&v.label);
        }

        Self { label: label.into(), source: source.into(), idents }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BindGroupDesc {
    pub entries: Vec<wgpu::BindGroupLayoutEntry>,
    // Name for group preprocessor
    pub label: CowStr,
}
impl SyncAssetKey<Arc<wgpu::BindGroupLayout>> for BindGroupDesc {
    fn load(&self, assets: AssetCache) -> Arc<wgpu::BindGroupLayout> {
        let gpu = GpuKey.get(&assets);

        let layout =
            gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some(&*self.label), entries: &self.entries });

        Arc::new(layout)
    }
}

/// Represents a shader and its layout
pub struct Shader {
    module: wgpu::ShaderModule,
    source: Option<String>,
    // Ordered sets
    bind_group_layouts: Vec<Arc<wgpu::BindGroupLayout>>,
    bind_group_labels: Vec<String>,
    pub idents: HashMap<CowStr, WgslValue>,
    label: CowStr,
}

impl Shader {
    pub fn from_modules<'a>(
        assets: &AssetCache,
        label: impl Into<CowStr>,
        modules: impl IntoIterator<Item = &'a ShaderModule>,
    ) -> Arc<Self> {
        let label = label.into();
        let gpu = GpuKey.get(assets);

        let mut idents: HashMap<CowStr, WgslValue> = HashMap::new();
        let mut bind_group_layouts = Vec::new();
        let mut bind_group_labels = Vec::new();

        #[allow(unstable_name_collisions)]
        let mut source: String = modules
            .into_iter()
            .flat_map(|module| {
                for ident in module.idents.iter() {
                    match ident {
                        ShaderModuleIdentifier::BindGroup(desc) => {
                            // Allocate group
                            let layout = desc.load(assets.clone());

                            if idents.insert(desc.label.clone(), WgslValue::Int32(bind_group_layouts.len() as u32)).is_some() {
                                panic!("Duplicate bind group {}", desc.label);
                            }

                            bind_group_layouts.push(layout);
                            bind_group_labels.push(desc.label.to_string());
                        }
                        ShaderModuleIdentifier::Constant { name, value } => {
                            if idents.insert(name.clone(), value.clone()).is_some() {
                                panic!("Redefined constant {name}={value:?} in {}", module.label);
                            }
                        }
                    }
                }

                module.source.lines()
            })
            .filter(|line| !line.starts_with("//"))
            .intersperse("\n")
            .collect();

        for (key, value) in idents.iter() {
            source = source.replace(&format!("#{key}"), &value.to_wgsl());
        }

        #[cfg(debug_assertions)]
        {
            std::fs::create_dir_all("tmp/").unwrap();
            std::fs::write(format!("tmp/{label}.wgsl"), source.as_bytes()).unwrap();
        }
        #[cfg(debug_assertions)]
        let src = Some(source.to_string());
        #[cfg(not(debug_assertions))]
        let src = None;

        let module = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor { label: Some(&label), source: wgpu::ShaderSource::Wgsl(source.into()) });

        Arc::new(Self { module, bind_group_layouts, bind_group_labels, idents, source: src, label })
    }

    pub fn ref_layouts(&self) -> Vec<&wgpu::BindGroupLayout> {
        self.bind_group_layouts.iter().map(|v| &**v).collect_vec()
    }
    pub fn layouts(&self) -> &[Arc<wgpu::BindGroupLayout>] {
        &self.bind_group_layouts
    }

    pub fn get_bind_group_layout_by_name(&self, name: &str) -> Option<&Arc<wgpu::BindGroupLayout>> {
        self.idents.get(name).and_then(WgslValue::as_integer).and_then(|v| self.bind_group_layouts.get(v as usize))
    }

    pub fn get_bind_group_index_by_name(&self, name: &str) -> Option<u32> {
        self.idents.get(name).and_then(WgslValue::as_integer)
    }

    pub fn module(&self) -> &wgpu::ShaderModule {
        &self.module
    }

    pub fn source(&self) -> Option<&String> {
        self.source.as_ref()
    }

    pub fn bind_all<'a>(&self, computepass: &mut wgpu::ComputePass<'a>, binding_context: &HashMap<String, &'a wgpu::BindGroup>) {
        for (index, id) in self.bind_group_labels.iter().enumerate() {
            computepass.set_bind_group(index as u32, binding_context.get(id).unwrap(), &[]);
        }
    }

    pub fn to_pipeline(self: &Arc<Self>, gpu: &Gpu, info: GraphicsPipelineInfo) -> GraphicsPipeline {
        let layout = gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{}.layout", self.label)),
            bind_group_layouts: &self.ref_layouts(),
            push_constant_ranges: &[],
        });

        let pipeline = gpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("{}.pipeline", self.label)),
            layout: Some(&layout),
            vertex: wgpu::VertexState { module: self.module(), entry_point: info.vs_main, buffers: &[] },
            primitive: wgpu::PrimitiveState {
                front_face: info.front_face,
                cull_mode: info.cull_mode,
                topology: info.topology,
                ..Default::default()
            },
            fragment: Some(wgpu::FragmentState { module: self.module(), entry_point: info.fs_main, targets: info.targets }),
            depth_stencil: info.depth,
            multisample: wgpu::MultisampleState { count: DEFAULT_SAMPLE_COUNT, mask: !0, alpha_to_coverage_enabled: false },
            multiview: None,
        });

        GraphicsPipeline { pipeline, shader: self.clone() }
    }

    pub fn to_compute_pipeline(self: &Arc<Self>, gpu: &Gpu, entry_point: &str) -> ComputePipeline {
        let layout = gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{}.compute_layout", self.label)),
            bind_group_layouts: &self.ref_layouts(),
            push_constant_ranges: &[],
        });

        let pipeline = gpu.device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some(&format!("{}.compute_pipeline", self.label)),
            layout: Some(&layout),
            module: self.module(),
            entry_point,
        });

        ComputePipeline { pipeline, shader: self.clone() }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphicsPipelineInfo<'a> {
    pub vs_main: &'a str,
    pub fs_main: &'a str,
    pub depth: Option<wgpu::DepthStencilState>,
    pub targets: &'a [Option<wgpu::ColorTargetState>],
    pub front_face: wgpu::FrontFace,
    pub cull_mode: Option<wgpu::Face>,
    pub topology: wgpu::PrimitiveTopology,
}

impl<'a> Default for GraphicsPipelineInfo<'a> {
    fn default() -> Self {
        Self {
            vs_main: "vs_main",
            fs_main: "fs_main",
            depth: None,
            targets: &[],
            front_face: wgpu::FrontFace::Cw,
            cull_mode: None,
            topology: wgpu::PrimitiveTopology::TriangleList,
        }
    }
}

pub type GraphicsPipeline = Pipeline<wgpu::RenderPipeline>;
pub type ComputePipeline = Pipeline<wgpu::ComputePipeline>;
pub struct Pipeline<P> {
    pipeline: P,
    shader: Arc<Shader>,
}

impl<P> Pipeline<P> {
    /// Get a reference to the graphics pipeline's pipeline.
    pub fn pipeline(&self) -> &P {
        &self.pipeline
    }

    /// Get a reference to the pipeline's shader.
    #[must_use]
    pub fn shader(&self) -> &Shader {
        self.shader.as_ref()
    }
}

impl ComputePipeline {
    /// Panics if a bind group does not exist
    pub fn bind<'a>(&'a self, renderpass: &mut wgpu::ComputePass<'a>, name: &str, bind_group: &'a wgpu::BindGroup) {
        let id = match self.shader.get_bind_group_index_by_name(name) {
            Some(v) => v,
            None => {
                panic!("Missing bind group {name:?}");
            }
        };
        renderpass.set_bind_group(id, bind_group, &[]);
    }
}

impl GraphicsPipeline {
    /// Panics if a bind group does not exist
    pub fn bind<'a>(&'a self, renderpass: &mut wgpu::RenderPass<'a>, name: &str, bind_group: &'a wgpu::BindGroup) {
        let id = match self.shader.get_bind_group_index_by_name(name) {
            Some(v) => v,
            None => {
                panic!("Missing bind group {name:?}");
            }
        };
        renderpass.set_bind_group(id, bind_group, &[]);
    }
}

impl<P> std::ops::Deref for Pipeline<P> {
    type Target = Shader;

    fn deref(&self) -> &Self::Target {
        &self.shader
    }
}

impl<'a> GraphicsPipelineInfo<'a> {
    pub fn with_depth(self) -> GraphicsPipelineInfo<'a> {
        Self {
            depth: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                // This is Greater because we're using reverse-z NDC
                depth_compare: wgpu::CompareFunction::Greater,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            ..self
        }
    }

    pub fn with_depth_bias(mut self, state: DepthBiasState) -> GraphicsPipelineInfo<'a> {
        self.depth.as_mut().expect("Attempt to set depth bias without a depth buffer").bias = state;
        self
    }
}
