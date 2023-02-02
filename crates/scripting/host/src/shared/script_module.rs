use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Context;
use async_trait::async_trait;
use elements_ecs::{EntityId, EntityUid, World};
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
        external_component_ids: HashSet<String>,
        enabled: bool,
    ) -> Self {
        ScriptModule {
            files: HashMap::new(),
            description: description.into(),
            enabled,
            external_component_ids,
        }
    }

    pub fn migrate_ids(&mut self, _old_to_new_ids: &HashMap<EntityId, EntityId>) {}

    pub fn files(&self) -> &HashMap<PathBuf, File> {
        &self.files
    }

    pub fn populate_files(&mut self, name: &str, scripting_interface: &str) {
        for (filename, contents) in Self::STATIC_FILE_TEMPLATES {
            let filename = PathBuf::from(filename);
            let contents = contents
                .replace("{{name}}", &util::sanitize(&name))
                .replace("{{description}}", &self.description)
                .replace("{{scripting_interface}}", scripting_interface);
            let file = File::new_at_now(contents);

            self.files.entry(filename).or_insert(file);
        }
    }

    /// Ignores system-controlled files
    pub fn insert_multiple(
        &mut self,
        module_name: &str,
        scripting_interfaces: &[&str],
        primary_scripting_interface: &str,
        files: &FileMap,
    ) -> anyhow::Result<()> {
        for (relative_path, new_file) in files {
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
        let relative_path = elements_std::path::normalize(relative_path);

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
        self.files.remove(&relative_path);
    }

    pub fn enabled(&self) -> bool {
        self.enabled
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

                #[main]
                pub async fn main() -> EventResult {
                    EventOk
                }
            "#},
        ),
    ];
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
    #[serde(default)]
    pub external_component_ids: HashSet<String>,
}
impl ScriptModuleBundle {
    pub fn to_json(name: &str, sm: &ScriptModule) -> String {
        let files = sm.files().clone();
        serde_json::to_string_pretty(&ScriptModuleBundle {
            name: name.to_owned(),
            files,
            description: sm.description.clone(),
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
