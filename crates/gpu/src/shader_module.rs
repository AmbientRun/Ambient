use std::{
    char::ParseCharError,
    collections::{btree_map, BTreeMap, BTreeSet, HashMap},
    sync::Arc,
};

use aho_corasick::AhoCorasick;
use ambient_std::{asset_cache::*, CowStr};
use itertools::Itertools;
use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, ComputePipelineDescriptor, DepthBiasState, ShaderStages,
};

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
pub struct ShaderIdent {
    name: CowStr,
    value: WgslValue,
}

impl ShaderIdent {
    /// Shortcut for unescaped text replacement
    pub fn raw(name: impl Into<CowStr>, value: impl Into<CowStr>) -> Self {
        Self { name: name.into(), value: WgslValue::Raw(value.into()) }
    }

    /// Replaces any occurence of `name` with the wgsl representation of `value`
    pub fn constant(name: impl Into<CowStr>, value: impl Into<WgslValue>) -> Self {
        Self { name: name.into(), value: value.into() }
    }
}

type BindingEntry = (CowStr, BindGroupLayoutEntry);

/// Defines a part of a shader, with preprocessing.
///
/// Each shadermodule contains:
/// - Source code
/// - Dependencies
/// - Identifier, used for preprocessing and replacing, such as constants
/// - A list of binding entries for generating the complete pipeline layout when the shader is assembled.
///     The bindings *do not* describe complete binding groups, as they may be spread out over several shader modules.
///
///     As such, it is not possible to get the bind group layout from a single shader module. Prefer to split out and reuse the entries in a separate function
#[derive(Debug, Default)]
pub struct ShaderModule {
    /// The unique name of the shadermodule.
    pub name: CowStr,
    /// The wgsl source for the module, *without* dependencies
    pub source: CowStr,

    /// Dependencies for the module
    pub dependencies: Vec<Arc<ShaderModule>>,

    // Use the label to preprocess constants
    pub idents: Vec<ShaderIdent>,
    bindings: Vec<BindingEntry>,
}

impl ShaderModule {
    pub fn new(name: impl Into<CowStr>, source: impl Into<CowStr>) -> Self {
        Self {
            name: name.into(),
            source: source.into(),
            idents: Default::default(),
            bindings: Default::default(),
            dependencies: Default::default(),
        }
    }

    pub fn with_ident(mut self, ident: ShaderIdent) -> Self {
        self.idents.push(ident);
        self
    }

    pub fn with_binding(mut self, group: impl Into<CowStr>, entry: BindGroupLayoutEntry) -> Self {
        self.bindings.push((group.into(), entry));
        self
    }

    pub fn with_bindings(mut self, bindings: impl IntoIterator<Item = (CowStr, BindGroupLayoutEntry)>) -> Self {
        self.bindings.extend(bindings.into_iter());
        self
    }

    pub fn with_binding_desc(mut self, desc: BindGroupDesc) -> Self {
        let group = desc.label.clone();
        self.bindings.extend(desc.entries.iter().map(|&entry| (group.clone(), entry)));
        self
    }

    pub fn with_dependency(mut self, module: Arc<ShaderModule>) -> Self {
        self.dependencies.push(module);
        self
    }

    pub fn with_dependencies(mut self, modules: impl IntoIterator<Item = Arc<ShaderModule>>) -> Self {
        self.dependencies.extend(modules);
        self
    }

    fn sanitized_label(&self) -> String {
        self.name.replace(|v: char| !v.is_ascii_alphanumeric() && !"_-.".contains(v), "?")
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
        tracing::info!("Creating bind group: {self:#?}");
        let gpu = GpuKey.get(&assets);

        let layout =
            gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some(&*self.label), entries: &self.entries });

        Arc::new(layout)
    }
}

/// Returns all shader modules in the dependency graph in topological order
///
/// # Panics
///
/// If the dependency graph contains a cycle
fn resolve_module_graph<'a>(roots: impl IntoIterator<Item = &'a ShaderModule>) -> Vec<&'a ShaderModule> {
    enum VisitedState {
        Pending,
        Visited,
    }

    let mut visited = BTreeMap::new();

    fn visit<'a>(
        visited: &mut BTreeMap<&'a str, VisitedState>,
        result: &mut Vec<&'a ShaderModule>,
        module: &'a ShaderModule,
        backtrace: &[&str],
    ) {
        match visited.entry(&module.name) {
            btree_map::Entry::Vacant(slot) => {
                slot.insert(VisitedState::Pending);
            }
            btree_map::Entry::Occupied(slot) => match slot.get() {
                VisitedState::Pending => panic!("Circular dependency for module: {:?} in {:?}", module.name, backtrace),
                VisitedState::Visited => return,
            },
        }

        let backtrace = backtrace.iter().copied().chain([&*module.name]).collect_vec();

        // Ensure dependencies are satisfied first
        for module in &module.dependencies {
            visit(visited, result, module, &backtrace)
        }

        visited.insert(&module.name, VisitedState::Visited);

        result.push(module);
    }

    let mut result = Vec::new();
    for root in roots {
        visit(&mut visited, &mut result, root, &[]);
    }

    result
}

/// Represents a shader and its layout
pub struct Shader {
    module: wgpu::ShaderModule,
    // Ordered sets
    bind_group_layouts: Vec<Arc<wgpu::BindGroupLayout>>,
    bind_group_index: BTreeMap<String, usize>,
    label: CowStr,
}

impl Shader {
    pub fn from_modules(assets: &AssetCache, label: impl Into<CowStr>, module: &ShaderModule) -> Arc<Self> {
        let label = label.into();
        let gpu = GpuKey.get(assets);

        let _span = tracing::info_span!("Shader::from_modules", ?label).entered();

        // The complete dependency graph, in the correct order
        let modules = resolve_module_graph([module]);

        tracing::info!("Compiling shader from modules: {:#?}", modules);

        // Resolve all bind groups, resolving the names to an index
        let mut bind_group_index = BTreeMap::new();
        let mut bind_groups = Vec::new();
        for module in &modules {
            for binding in &module.bindings {
                let index = *bind_group_index.entry(binding.0.as_ref().to_owned()).or_insert_with(|| {
                    let group = binding.0.clone();
                    let index = bind_groups.len();

                    tracing::info!("Resolved bind group: {group}:{index}");

                    bind_groups.push(BindGroupDesc { label: group, entries: Default::default() });
                    index
                });

                let desc = &mut bind_groups[index];
                desc.entries.push(binding.1);
            }
        }

        // Now for the fun part: constructing the binding group layout descriptors
        let bind_group_layouts = bind_groups.iter().map(|desc| desc.get(assets)).collect_vec();

        // Efficiently replace all identifiers
        let (patterns, replace_with): (Vec<_>, Vec<_>) = modules
            .iter()
            .flat_map(|v| v.idents.iter().map(|ShaderIdent { name, value }| (format!("#{name}"), value.to_wgsl())))
            .chain(bind_group_index.iter().map(|(name, &index)| (format!("#{name}"), (index as u32).to_string())))
            .unzip();

        tracing::info!("Preprocessing shader using {patterns:?} => {replace_with:?}");

        // Collect the raw source code
        let source = {
            let source = modules
                .iter()
                .map(|module| {
                    let div = "--------------------------------";
                    let label = module.sanitized_label();
                    let source = &module.source;
                    format!("// {div}\n// @module: {label}\n// {div}\n{source}")
                })
                .join("\n\n");

            AhoCorasick::new(patterns).replace_all(&source, &replace_with)
        };

        // let mut source: String = modules
        //     .by_ref()
        //     .map(|module| {
        //         for ident in module.idents.iter() {
        //             match ident {
        //                 ShaderModuleIdentifier::BindGroup(desc) => {
        //                     // Allocate group
        //                     let layout = desc.load(assets.clone());

        //                     if idents.insert(desc.label.clone(), WgslValue::Int32(bind_group_layouts.len() as u32)).is_some() {
        //                         panic!("Duplicate bind group {}", desc.label);
        //                     }

        //                     bind_group_layouts.push(layout);
        //                     bind_group_labels.push(desc.label.to_string());
        //                 }
        //                 ShaderModuleIdentifier::Constant { name, value } => {
        //                     if idents.insert(name.clone(), value.clone()).is_some() {
        //                         panic!("Redefined constant {name}={value:?} in {}", module.label);
        //                     }
        //                 }
        //             }
        //         }
        //         module_names.push(&module.label);

        //         let div = "--------------------------------";
        //         let label = module.sanitized_label();
        //         let source = &module.source;
        //         format!("// {div}\n// @module: {label}\n// {div}\n{source}")
        //     })
        //     .join("\n\n");

        // tracing::info!("Using modules: {module_names:#?}");

        // for (key, value) in idents.iter() {
        //     source = source.replace(&format!("#{key}"), &value.to_wgsl());
        // }

        #[cfg(all(not(target_os = "unknown"), debug_assertions))]
        {
            let path = format!("tmp/{label}.wgsl");
            eprintln!("Writing shader to {path}");
            std::fs::create_dir_all("tmp/").unwrap();
            std::fs::write(path, source.as_bytes()).unwrap();
        }

        let module = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor { label: Some(&label), source: wgpu::ShaderSource::Wgsl(source.into()) });

        Arc::new(Self { module, bind_group_layouts, bind_group_index, label })
    }

    pub fn ref_layouts(&self) -> Vec<&BindGroupLayout> {
        self.bind_group_layouts.iter().map(|v| &**v).collect_vec()
    }
    pub fn layouts(&self) -> &[Arc<BindGroupLayout>] {
        &self.bind_group_layouts
    }

    pub fn get_bind_group_layout_by_name(&self, name: &str) -> Option<&BindGroupLayout> {
        self.bind_group_index.get(name).map(|&v| &*self.bind_group_layouts[v])
    }

    pub fn get_bind_group_index_by_name(&self, name: &str) -> Option<u32> {
        self.bind_group_index.get(name).map(|&v| v as _)
    }

    /// The wgpu shader module
    pub fn module(&self) -> &wgpu::ShaderModule {
        &self.module
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
