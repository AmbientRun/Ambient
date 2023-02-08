use std::{marker::PhantomData, sync::Arc};

use anyhow::Context;
use async_trait::async_trait;
use elements_ecs::World;
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use wasi_common::{
    file::{FdFlags, FileType},
    WasiCtx, WasiFile,
};

use super::{
    bindings,
    guest_conversion::GuestConvert,
    host_guest_state::GetBaseHostGuestState,
    interface::guest::{Guest, GuestData, RunContext},
    ScriptContext,
};

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

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ScriptModuleErrors {
    pub compiletime: Vec<String>,
    pub runtime: Vec<String>,
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
