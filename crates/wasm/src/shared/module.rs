use std::{any::Any, collections::HashSet, sync::Arc};

use ambient_ecs::{EntityId, World};
use data_encoding::BASE64;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use super::{
    bindings::{self, BindingsBound},
    borrowed_types::ValueBorrow,
    wit, RunContext,
};

#[derive(Clone)]
pub struct ModuleBytecode(pub Vec<u8>);
impl std::fmt::Debug for ModuleBytecode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ModuleBytecode")
            .field(&BASE64.encode(&self.0))
            .finish()
    }
}
impl std::fmt::Display for ModuleBytecode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ModuleBytecode({} bytes)", self.0.len())
    }
}
impl Serialize for ModuleBytecode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&BASE64.encode(&self.0))
    }
}
impl<'de> Deserialize<'de> for ModuleBytecode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, Visitor};

        struct ModuleBytecodeVisitor;
        impl<'de> Visitor<'de> for ModuleBytecodeVisitor {
            type Value = ModuleBytecode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a base64-encoded string of bytes")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                BASE64
                    .decode(v.as_bytes())
                    .map_err(|err| {
                        E::custom(format!("failed to decode base64-encoded string: {err}"))
                    })
                    .map(ModuleBytecode)
            }
        }

        deserializer.deserialize_str(ModuleBytecodeVisitor)
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ModuleErrors {
    pub compiletime: Vec<String>,
    pub runtime: Vec<String>,
}

struct WasmContext<Bindings: BindingsBound> {
    wasi: ambient_wasmtime_wasi::WasiCtx,
    bindings: Bindings,
}

pub trait ModuleStateBehavior: Sync + Send {
    fn run(&mut self, world: &mut World, context: &RunContext) -> anyhow::Result<()>;
    fn drain_spawned_entities(&mut self) -> HashSet<EntityId>;
    fn supports_event(&self, event_name: &str) -> bool;
}

pub type Messenger = Box<dyn Fn(&World, &str) + Sync + Send>;

pub struct ModuleStateArgs<'a> {
    pub component_bytecode: &'a [u8],
    pub stdout_output: Messenger,
    pub stderr_output: Messenger,
}

#[derive(Clone)]
pub struct ModuleState {
    // Wrap the inner state to make it easily clonable and to allow for erasing
    // the precise bindings in use
    inner: Arc<RwLock<dyn ModuleStateBehavior>>,
}
impl ModuleState {
    fn new<Bindings: BindingsBound + 'static>(
        args: ModuleStateArgs<'_>,
        bindings: Bindings,
    ) -> anyhow::Result<Self> {
        let ModuleStateArgs {
            component_bytecode,
            stdout_output,
            stderr_output,
        } = args;

        Ok(Self {
            inner: Arc::new(RwLock::new(ModuleStateInnerImpl::new(
                component_bytecode,
                stdout_output,
                stderr_output,
                bindings,
            )?)),
        })
    }

    pub fn create_state_maker<Bindings: BindingsBound + 'static>(
        bindings: Bindings,
    ) -> Arc<dyn Fn(ModuleStateArgs<'_>) -> anyhow::Result<Self> + Sync + Send> {
        Arc::new(move |args: ModuleStateArgs<'_>| Self::new(args, bindings.clone()))
    }
}
impl ModuleStateBehavior for ModuleState {
    fn run(&mut self, world: &mut World, context: &RunContext) -> anyhow::Result<()> {
        self.inner.write().run(world, context)
    }

    fn drain_spawned_entities(&mut self) -> HashSet<EntityId> {
        self.inner.write().drain_spawned_entities()
    }

    fn supports_event(&self, event_name: &str) -> bool {
        self.inner.read().supports_event(event_name)
    }
}

struct ModuleStateInnerImpl<Bindings: BindingsBound> {
    _engine: wasmtime::Engine,
    store: wasmtime::Store<WasmContext<Bindings>>,

    guest_bindings: wit::Bindings,
    _guest_instance: wasmtime::component::Instance,

    stdout_consumer: WasiOutputStreamConsumer,
    stderr_consumer: WasiOutputStreamConsumer,
}

impl<Bindings: BindingsBound> std::fmt::Debug for ModuleStateInnerImpl<Bindings> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModuleStateInner").finish()
    }
}
impl<Bindings: BindingsBound> ModuleStateInnerImpl<Bindings> {
    fn new(
        component_bytecode: &[u8],
        stdout_output: Box<dyn Fn(&World, &str) + Sync + Send>,
        stderr_output: Box<dyn Fn(&World, &str) + Sync + Send>,
        bindings: Bindings,
    ) -> anyhow::Result<Self> {
        let mut config = wasmtime::Config::new();
        config.wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable);
        config.wasm_component_model(true);
        let engine = wasmtime::Engine::new(&config)?;

        let (stdout_output, stdout_consumer) = WasiOutputStream::make(stdout_output);
        let (stderr_output, stderr_consumer) = WasiOutputStream::make(stderr_output);
        let mut store = wasmtime::Store::new(
            &engine,
            WasmContext {
                wasi: ambient_wasmtime_wasi::WasiCtxBuilder::new()
                    .stdout(stdout_output)
                    .stderr(stderr_output)
                    .build(),
                bindings,
            },
        );

        let mut linker = wasmtime::component::Linker::<WasmContext<Bindings>>::new(&engine);
        ambient_wasmtime_wasi::add_to_linker(&mut linker, |x| &mut x.wasi)?;
        wit::Bindings::add_to_linker(&mut linker, |x| &mut x.bindings)?;

        let component = wasmtime::component::Component::from_binary(&engine, component_bytecode)?;

        let (guest_bindings, guest_instance) =
            wit::Bindings::instantiate(&mut store, &component, &linker)?;

        // Initialise the runtime.
        guest_bindings
            .guest()
            .call_init(&mut store, wit::shared::INTERFACE_VERSION)?;

        Ok(Self {
            _engine: engine,
            store,
            guest_bindings,
            _guest_instance: guest_instance,

            stdout_consumer,
            stderr_consumer,
        })
    }
}
impl<Bindings: BindingsBound> ModuleStateBehavior for ModuleStateInnerImpl<Bindings> {
    fn run(&mut self, world: &mut World, context: &RunContext) -> anyhow::Result<()> {
        let RunContext {
            event_name,
            event_data,
            time,
        } = context;

        self.store.data_mut().bindings.base_mut().set_world(world);

        let components = bindings::convert_entity_data_to_components(event_data);
        let components: Vec<_> = components
            .iter()
            .map(|(k, v)| (*k, ValueBorrow::from(v)))
            .collect();
        let components: Vec<_> = components.iter().map(|(k, v)| (*k, v.as_wit())).collect();
        self.guest_bindings
            .guest()
            .call_exec(&mut self.store, *time, event_name, &components)?;

        self.store.data_mut().bindings.base_mut().clear_world();

        self.stdout_consumer.process_incoming(world);
        self.stderr_consumer.process_incoming(world);

        Ok(())
    }

    fn drain_spawned_entities(&mut self) -> HashSet<EntityId> {
        std::mem::take(&mut self.store.data_mut().bindings.base_mut().spawned_entities)
    }

    fn supports_event(&self, event_name: &str) -> bool {
        self.store
            .data()
            .bindings
            .base()
            .subscribed_events
            .contains(event_name)
    }
}

struct WasiOutputStream(flume::Sender<String>);
impl WasiOutputStream {
    fn make(
        outputter: Box<dyn Fn(&World, &str) + Sync + Send>,
    ) -> (Box<Self>, WasiOutputStreamConsumer) {
        let (tx, rx) = flume::unbounded();
        (
            Box::new(Self(tx)),
            WasiOutputStreamConsumer { rx, outputter },
        )
    }
}

#[async_trait::async_trait]
impl wasi_common::OutputStream for WasiOutputStream {
    fn as_any(&self) -> &dyn Any {
        self
    }

    async fn writable(&self) -> Result<(), wasi_common::Error> {
        Ok(())
    }

    async fn write(&mut self, buf: &[u8]) -> Result<u64, wasi_common::Error> {
        let msg = std::str::from_utf8(buf)
            .map_err(|e| wasi_common::Error::trap(e.into()))?
            .trim();
        self.0
            .send(msg.to_string())
            .map_err(|e| wasi_common::Error::trap(e.into()))?;
        Ok(buf.len().try_into()?)
    }
}

struct WasiOutputStreamConsumer {
    rx: flume::Receiver<String>,
    outputter: Box<dyn Fn(&World, &str) + Sync + Send>,
}
impl WasiOutputStreamConsumer {
    fn process_incoming(&self, world: &World) {
        for msg in self.rx.drain() {
            (self.outputter)(world, &msg);
        }
    }
}
