use std::{
    collections::{HashMap, HashSet},
    fmt::{Display, Write},
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Context;
use async_trait::async_trait;
use elements_ecs::{with_component_registry, EntityId, EntityUid, World};
use elements_std::asset_url::ObjectRef;
use glam::Vec3;
use indexmap::IndexMap;
use indoc::indoc;
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use wasi_common::{
    file::{FdFlags, FileType},
    WasiCtx, WasiFile,
};

use super::{
    bindings,
    dependencies::{self, clean_cargo_toml},
    guest_conversion::GuestConvert,
    host_guest_state::GetBaseHostGuestState,
    interface::guest::{Guest, GuestData, RunContext},
    util, ScriptContext,
};

pub type FileMap = HashMap<PathBuf, File>;
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ScriptModule {
    files: FileMap,
    pub description: String,
    pub parameters: ParametersMap,
    last_updated_by_parameters: bool,
    pub external_component_ids: HashSet<String>,
    pub enabled: bool,
}
impl Display for ScriptModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ScriptModule")
    }
}
impl ScriptModule {
    pub fn new(
        description: impl Into<String>,
        parameters: ParametersMap,
        external_component_ids: HashSet<String>,
        enabled: bool,
    ) -> Self {
        ScriptModule {
            files: HashMap::new(),
            description: description.into(),
            parameters,
            enabled,
            external_component_ids,
            last_updated_by_parameters: false,
        }
    }

    pub fn migrate_ids(&mut self, _old_to_new_ids: &HashMap<EntityId, EntityId>) {}

    pub fn files(&self) -> &HashMap<PathBuf, File> {
        &self.files
    }

    pub fn system_controlled_files() -> HashSet<PathBuf> {
        ["src/params.rs", "src/components.rs"]
            .into_iter()
            .map(|p| p.into())
            .collect()
    }

    pub fn populate_files(&mut self, name: &str, scripting_interface: &str) {
        self.regenerate_params_file(scripting_interface);
        self.regenerate_components_file(scripting_interface);
        for (filename, contents) in Self::STATIC_FILE_TEMPLATES {
            let filename = PathBuf::from(filename);
            let contents = contents
                .replace("{{name}}", &util::sanitize(&name))
                .replace("{{description}}", &self.description)
                .replace("{{scripting_interface}}", scripting_interface);
            let file = File::new_at_now(contents);

            self.files.entry(filename).or_insert(file);
        }
        self.last_updated_by_parameters = false;
    }

    pub fn update_parameters(&mut self, parameters: ParametersMap, scripting_interface: &str) {
        self.parameters = parameters;
        self.last_updated_by_parameters = true;
        self.regenerate_params_file(scripting_interface);
        self.regenerate_components_file(scripting_interface);
    }

    /// Ignores system-controlled files
    pub fn insert_multiple(
        &mut self,
        module_name: &str,
        scripting_interfaces: &[&str],
        primary_scripting_interface: &str,
        files: &FileMap,
    ) -> anyhow::Result<()> {
        let system_controlled_files = Self::system_controlled_files();
        for (relative_path, new_file) in files {
            if system_controlled_files.contains(relative_path) {
                continue;
            }
            self.insert(scripting_interfaces, relative_path, new_file)?;
        }
        self.populate_files(module_name, primary_scripting_interface);
        Ok(())
    }

    pub fn insert(
        &mut self,
        scripting_interfaces: &[&str],
        relative_path: &Path,
        new_file: &File,
    ) -> anyhow::Result<()> {
        let relative_path = elements_std::path::normalize(&relative_path);
        if ScriptModule::system_controlled_files().contains(&relative_path) {
            anyhow::bail!("{relative_path:?} is system-controlled and cannot be updated");
        }

        if relative_path == Path::new("Cargo.toml") {
            if let Some(old_cargo) = self.files.get(Path::new("Cargo.toml")) {
                self.files.insert(
                    relative_path,
                    File::new_at_now(dependencies::merge_cargo_toml(
                        scripting_interfaces,
                        &old_cargo.contents,
                        &new_file.contents,
                    )?),
                );
            } else {
                self.files.insert(
                    relative_path,
                    File::new_at_now(clean_cargo_toml(scripting_interfaces, &new_file.contents)?),
                );
            }
        } else {
            self.files.insert(relative_path, new_file.clone());
        }

        Ok(())
    }

    pub fn remove(&mut self, relative_path: &Path) {
        let relative_path = elements_std::path::normalize(relative_path);
        if ScriptModule::system_controlled_files()
            .iter()
            .any(|pb| pb == &relative_path)
        {
            return;
        }
        self.files.remove(&relative_path);
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn last_updated_by_parameters(&self) -> bool {
        self.last_updated_by_parameters
    }
}
impl ScriptModule {
    const STATIC_FILE_TEMPLATES: &[(&'static str, &'static str)] = &[
        (
            "Cargo.toml",
            indoc! {r#"
                [package]
                edition = "2021"
                name = "{{name}}"
                description = "{{description}}"
                version = "0.1.0"

                [lib]
                crate-type = ["cdylib"]

                [dependencies]
                {{scripting_interface}} = {path = "../../interfaces/{{scripting_interface}}"}
            "#},
        ),
        (
            "src/lib.rs",
            indoc! {r#"
                use {{scripting_interface}}::*;
                pub mod params;
                pub mod components;

                #[main]
                pub async fn main() -> EventResult {
                    EventOk
                }
            "#},
        ),
    ];

    fn regenerate_params_file(&mut self, scripting_interface: &str) {
        let mut contents = String::new();
        let _ = writeln!(contents, "#![allow(unused_imports)]");
        for (category, parameters) in &self.parameters {
            let category = category.trim().replace(' ', "_").to_lowercase();
            if category.is_empty() {
                continue;
            }

            let _ = writeln!(contents, "pub mod {category} {{");
            let _ = writeln!(contents, "    use {scripting_interface}::*;");
            for (key, value) in parameters {
                let key = key.trim().replace(' ', "_").to_uppercase();
                if key.is_empty() {
                    continue;
                }
                let value = match value {
                    Parameter::EntityUid(Some(uid)) => {
                        format!("EntityUid = EntityUid::new(\"{uid}\")")
                    }
                    Parameter::EntityUid(None) => continue,
                    Parameter::ObjectRef(url) => {
                        format!(r#"ObjectRef = ObjectRef::new("{url}")"#)
                    }
                    Parameter::Integer(v) => format!("i32 = {v}"),
                    Parameter::Float(v) => format!("f32 = {v} as f32"),
                    Parameter::Vec3(v) => {
                        format!(
                            "Vec3 = vec3({} as f32, {} as f32, {} as f32)",
                            v.x, v.y, v.z
                        )
                    }
                    Parameter::String(v) => format!(r#"&str = {v:?}"#),
                    Parameter::Bool(v) => format!("bool = {v}"),
                };

                let _ = writeln!(contents, "    pub const {key}: {value};");
            }
            let _ = writeln!(contents, "}}");
        }

        self.files
            .insert("src/params.rs".into(), File::new_at_now(contents));
    }

    fn regenerate_components_file(&mut self, scripting_interface: &str) {
        enum ComponentTreeNode {
            Category(HashMap<String, ComponentTreeNode>),
            Component { typename: &'static str, id: String },
        }
        impl Default for ComponentTreeNode {
            fn default() -> Self {
                ComponentTreeNode::Category(Default::default())
            }
        }
        impl ComponentTreeNode {
            fn insert(&mut self, id_portion: &str, id: &str, typename: &'static str) {
                if let ComponentTreeNode::Category(hm) = self {
                    let (prefix, suffix) = id_portion.split_once("::").unwrap_or(("", id_portion));
                    if prefix.is_empty() {
                        hm.insert(
                            suffix.to_string(),
                            ComponentTreeNode::Component {
                                typename,
                                id: id.to_string(),
                            },
                        );
                    } else {
                        hm.entry(prefix.to_string())
                            .or_default()
                            .insert(suffix, id, typename);
                    }
                }
            }
        }

        let supported_types: HashMap<_, _> = bindings::SUPPORTED_COMPONENT_TYPES
            .iter()
            .copied()
            .collect();

        let mut root = ComponentTreeNode::default();
        with_component_registry(|registry| {
            for component in registry.all_external() {
                if let Some(typename) = supported_types.get(&component.type_id()) {
                    root.insert(&component.get_id(), &component.get_id(), typename);
                }
            }
        });

        fn write_to_file(
            output: &mut String,
            name: &str,
            component: &ComponentTreeNode,
            depth: usize,
            scripting_interface: &str,
        ) {
            let space = " ".repeat(depth * 4);
            match component {
                ComponentTreeNode::Category(hm) => {
                    if name.is_empty() {
                        for (key, value) in hm {
                            write_to_file(output, key, value, 0, scripting_interface);
                        }
                    } else {
                        writeln!(output, "{space}pub mod {name} {{").ok();
                        writeln!(output, "{space}    use {scripting_interface}::*;").ok();
                        for (key, value) in hm {
                            write_to_file(output, key, value, depth + 1, scripting_interface);
                        }
                        writeln!(output, "{space}}}").ok();
                    }
                }
                ComponentTreeNode::Component { typename, id, .. } => {
                    writeln!(
                        output,
                        r#"{space}static {}: LazyComponent<{typename}> = lazy_component!("{id}");"#,
                        name.to_uppercase()
                    )
                    .ok();
                    writeln!(
                        output,
                        "{space}pub fn {name}() -> Component<{typename}> {{ *{} }}",
                        name.to_uppercase()
                    )
                    .ok();
                }
            }
        }
        let mut contents = String::new();
        let _ = writeln!(contents, "#![allow(unused_imports)]");
        write_to_file(&mut contents, "", &root, 0, scripting_interface);

        self.files
            .insert("src/components.rs".into(), File::new_at_now(contents));
    }
}

#[derive(Clone)]
pub struct ScriptModuleBytecode(pub Vec<u8>);
impl std::fmt::Debug for ScriptModuleBytecode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ScriptModuleBytecode")
            .field(&base64::encode(&self.0))
            .finish()
    }
}
impl std::fmt::Display for ScriptModuleBytecode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ScriptModuleBytecode({} bytes)", self.0.len())
    }
}
impl Serialize for ScriptModuleBytecode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&base64::encode(&self.0))
    }
}
impl<'de> Deserialize<'de> for ScriptModuleBytecode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, Visitor};

        struct ScriptModuleBytecodeVisitor;
        impl<'de> Visitor<'de> for ScriptModuleBytecodeVisitor {
            type Value = ScriptModuleBytecode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a base64-encoded string of bytes")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                base64::decode(v)
                    .map_err(|err| {
                        E::custom(format!("failed to decode base64-encoded string: {err}"))
                    })
                    .map(ScriptModuleBytecode)
            }
        }

        deserializer.deserialize_str(ScriptModuleBytecodeVisitor)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Parameter {
    EntityUid(Option<EntityUid>),
    ObjectRef(ObjectRef),
    Integer(i32),
    Float(f32),
    Vec3(Vec3),
    String(String),
    Bool(bool),
}
impl Default for Parameter {
    fn default() -> Self {
        Parameter::Integer(0)
    }
}

pub type ParametersMap = IndexMap<String, IndexMap<String, Parameter>>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct File {
    // TODO(mithun): consider using an enum of Plaintext(String)/Binary(Bytes) files so that people can include binary assets
    // in their crates
    pub contents: String,
    pub last_modified: chrono::DateTime<chrono::Utc>,
}
impl File {
    pub fn new_at_now(contents: String) -> Self {
        Self {
            contents,
            last_modified: chrono::Utc::now(),
        }
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ScriptModuleErrors {
    pub compiletime: Vec<String>,
    pub runtime: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScriptModuleBundle {
    pub name: String,
    pub files: FileMap,
    pub description: String,
    pub parameters: ParametersMap,
    #[serde(default)]
    pub external_component_ids: HashSet<String>,
}
impl ScriptModuleBundle {
    pub fn to_json(name: &str, sm: &ScriptModule) -> String {
        let mut files = sm.files().clone();
        for path in ScriptModule::system_controlled_files() {
            files.remove(&path);
        }
        serde_json::to_string_pretty(&ScriptModuleBundle {
            name: name.to_owned(),
            files,
            description: sm.description.clone(),
            parameters: sm.parameters.clone(),
            external_component_ids: sm.external_component_ids.clone(),
        })
        .unwrap()
    }
}

pub struct ScriptModuleState<
    Bindings: Send + Sync + 'static,
    Context: WasmContext<Bindings>,
    HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
> {
    _engine: wasmtime::Engine,
    store: Arc<Mutex<wasmtime::Store<Context>>>,

    guest_exports: Arc<Guest<Context>>,
    _guest_instance: wasmtime::Instance,

    _bindings: PhantomData<Bindings>,
    pub shared_state: Arc<RwLock<HostGuestState>>,
}
impl<
        Bindings: Send + Sync + 'static,
        Context: WasmContext<Bindings>,
        HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
    > Clone for ScriptModuleState<Bindings, Context, HostGuestState>
{
    fn clone(&self) -> Self {
        Self {
            _engine: self._engine.clone(),
            store: self.store.clone(),
            guest_exports: self.guest_exports.clone(),
            _guest_instance: self._guest_instance,
            _bindings: self._bindings,
            shared_state: self.shared_state.clone(),
        }
    }
}

impl<
        Bindings: Send + Sync + 'static,
        Context: WasmContext<Bindings>,
        HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
    > std::fmt::Debug for ScriptModuleState<Bindings, Context, HostGuestState>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScriptModuleState").finish()
    }
}
impl<
        Bindings: Send + Sync + 'static,
        Context: WasmContext<Bindings>,
        HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
    > ScriptModuleState<Bindings, Context, HostGuestState>
{
    pub fn new(
        bytecode: &[u8],
        stdout_output: Box<dyn Fn(&World, &str) + Sync + Send>,
        stderr_output: Box<dyn Fn(&World, &str) + Sync + Send>,
        make_wasm_context: impl Fn(WasiCtx, Arc<RwLock<HostGuestState>>) -> Context,
        add_to_linker: impl Fn(&mut wasmtime::Linker<Context>) -> anyhow::Result<()>,
        interface_version: u32,
    ) -> anyhow::Result<Self> {
        let shared_state = Arc::new(RwLock::new(HostGuestState::default()));

        let engine = wasmtime::Engine::default();
        let mut store = wasmtime::Store::new(
            &engine,
            make_wasm_context(
                wasmtime_wasi::sync::WasiCtxBuilder::new()
                    .stdout(Box::new(WasiOutputFile(
                        stdout_output,
                        shared_state.clone(),
                    )))
                    .stderr(Box::new(WasiOutputFile(
                        stderr_output,
                        shared_state.clone(),
                    )))
                    .build(),
                shared_state.clone(),
            ),
        );

        let (guest_exports, guest_instance) = {
            let mut linker: wasmtime::Linker<Context> = wasmtime::Linker::new(&engine);
            wasmtime_wasi::add_to_linker(&mut linker, |cx| &mut cx.base_wasm_context_mut().wasi)?;
            add_to_linker(&mut linker)?;

            let module = wasmtime::Module::from_binary(&engine, bytecode)?;
            Guest::instantiate(&mut store, &module, &mut linker, |cx| {
                &mut cx.base_wasm_context_mut().guest_data
            })?
        };

        // Initialise the runtime.
        guest_exports.init(&mut store)?;
        // Call the script's main function.
        guest_instance
            .get_func(&mut store, "call_main")
            .context("not a func")?
            .typed::<(u32,), (), _>(&store)?
            .call(&mut store, (interface_version,))?;

        Ok(Self {
            shared_state,
            _bindings: PhantomData,
            _engine: engine,
            store: Arc::new(Mutex::new(store)),
            guest_exports: Arc::new(guest_exports),
            _guest_instance: guest_instance,
        })
    }

    pub fn run(&mut self, world: &mut World, context: &ScriptContext) -> anyhow::Result<()> {
        let ScriptContext {
            event_name,
            event_data,
            time,
            frametime,
        } = context;

        self.shared_state().write().base_mut().set_world(world);

        // remap the generated entitydata to components to send across
        let components = bindings::convert_entity_data_to_components(event_data);
        // TEMPORARY: convert the host rep components to owned guest rep components
        let components: Vec<_> = components
            .iter()
            .map(|(id, ct)| (*id, ct.guest_convert()))
            .collect();
        // then get the borrowing representation
        // these two steps should be unnecessary once we can update to the component version of wit-bindgen
        let components: Vec<_> = components
            .iter()
            .map(|(id, ct)| (*id, ct.as_guest()))
            .collect();

        Ok(self.guest_exports.exec(
            &mut *self.store.lock(),
            RunContext {
                time: *time,
                frametime: *frametime,
            },
            event_name,
            &components,
        )?)
    }

    pub fn shared_state(&self) -> Arc<RwLock<HostGuestState>> {
        self.shared_state.clone()
    }
}

// TODO(mithun): come up with a more optimal way to do this that doesn't
// implicitly require unsafe and mutex locking
struct WasiOutputFile(
    Box<dyn Fn(&World, &str) + Sync + Send>,
    Arc<RwLock<dyn GetBaseHostGuestState + Sync + Send>>,
);
#[async_trait]
impl WasiFile for WasiOutputFile {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn get_filetype(&mut self) -> Result<FileType, wasmtime_wasi::Error> {
        Ok(FileType::Unknown)
    }

    async fn get_fdflags(&mut self) -> Result<FdFlags, wasmtime_wasi::Error> {
        Ok(FdFlags::APPEND)
    }

    async fn write_vectored<'a>(
        &mut self,
        bufs: &[std::io::IoSlice<'a>],
    ) -> Result<u64, wasmtime_wasi::Error> {
        let mut count = 0;
        for buf in bufs {
            if let Ok(text) = std::str::from_utf8(buf) {
                self.0(self.1.read().base().world(), text);
                count += text.len();
            }
        }
        Ok(count as u64)
    }
}

pub trait WasmContext<Bindings> {
    fn base_wasm_context_mut(&mut self) -> &mut BaseWasmContext;
}

pub struct BaseWasmContext {
    wasi: wasmtime_wasi::WasiCtx,
    guest_data: GuestData,
}
impl BaseWasmContext {
    pub fn new(wasi: wasmtime_wasi::WasiCtx) -> Self {
        Self {
            wasi,
            guest_data: Default::default(),
        }
    }
}
