use crate::shared;

use super::Source;
#[cfg(feature = "wit")]
use super::{bindings::BindingsBound, conversion::IntoBindgen};
use ambient_ecs::{EntityId, World};
use data_encoding::BASE64;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{any::Any, collections::HashSet, sync::Arc};
#[cfg(feature = "wit")]
use wasmtime_wasi::preview2 as wasi_preview2;

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
pub struct ModuleErrors(pub Vec<String>);

#[cfg(feature = "wit")]
struct WasmtimeContext<Bindings: BindingsBound> {
    bindings: Bindings,
    wasi: wasi_preview2::WasiCtx,
    table: wasi_preview2::Table,
}
#[cfg(feature = "wit")]
impl<B: BindingsBound> wasi_preview2::WasiView for WasmtimeContext<B> {
    fn table(&self) -> &wasi_preview2::Table {
        &self.table
    }

    fn table_mut(&mut self) -> &mut wasi_preview2::Table {
        &mut self.table
    }

    fn ctx(&self) -> &wasi_preview2::WasiCtx {
        &self.wasi
    }

    fn ctx_mut(&mut self) -> &mut wasi_preview2::WasiCtx {
        &mut self.wasi
    }
}

pub trait ModuleStateBehavior: Sync + Send {
    fn run(
        &mut self,
        world: &mut World,
        message_source: &Source,
        message_name: &str,
        message_data: &[u8],
    ) -> anyhow::Result<()>;
    fn drain_spawned_entities(&mut self) -> HashSet<EntityId>;
    fn listen_to_message(&mut self, event_name: String);
    fn supports_message(&self, event_name: &str) -> bool;
}

pub type Messenger = Box<dyn Fn(&World, &str) + Sync + Send>;

pub struct ModuleStateArgs<'a> {
    pub component_bytecode: &'a [u8],
    pub stdout_output: Messenger,
    pub stderr_output: Messenger,
    pub id: EntityId,
}

#[derive(Clone)]
pub struct ModuleState {
    // Wrap the inner state to make it easily clonable and to allow for erasing
    // the precise bindings in use
    inner: Arc<RwLock<dyn ModuleStateBehavior>>,
}
impl ModuleState {
    #[cfg(feature = "wit")]
    fn new<Bindings: BindingsBound + 'static>(
        args: ModuleStateArgs<'_>,
        bindings: fn(EntityId) -> Bindings,
    ) -> anyhow::Result<Self> {
        let ModuleStateArgs {
            component_bytecode,
            stdout_output,
            stderr_output,
            id,
        } = args;

        Ok(Self {
            inner: Arc::new(RwLock::new(ModuleStateInnerImpl::new(
                component_bytecode,
                stdout_output,
                stderr_output,
                bindings(id),
            )?)),
        })
    }

    #[cfg(feature = "wit")]
    pub fn create_state_maker<Bindings: BindingsBound + 'static>(
        bindings: fn(EntityId) -> Bindings,
    ) -> Arc<dyn Fn(ModuleStateArgs<'_>) -> anyhow::Result<ModuleState> + Send + Sync> {
        Arc::new(move |args: ModuleStateArgs<'_>| Self::new(args, bindings))
    }
}
impl ModuleStateBehavior for ModuleState {
    fn run(
        &mut self,
        world: &mut World,
        message_source: &Source,
        message_name: &str,
        message_data: &[u8],
    ) -> anyhow::Result<()> {
        self.inner
            .write()
            .run(world, message_source, message_name, message_data)
    }

    fn drain_spawned_entities(&mut self) -> HashSet<EntityId> {
        self.inner.write().drain_spawned_entities()
    }

    fn listen_to_message(&mut self, message_name: String) {
        self.inner.write().listen_to_message(message_name)
    }

    fn supports_message(&self, message_name: &str) -> bool {
        self.inner.read().supports_message(message_name)
    }
}

#[cfg(feature = "wit")]
struct ModuleStateInnerImpl<Bindings: BindingsBound> {
    store: wasmtime::Store<WasmtimeContext<Bindings>>,

    guest_bindings: shared::wit::Bindings,
    _guest_instance: wasmtime::component::Instance,

    stdout_consumer: WasiOutputStreamConsumer,
    stderr_consumer: WasiOutputStreamConsumer,
}

#[cfg(feature = "wit")]
impl<Bindings: BindingsBound> std::fmt::Debug for ModuleStateInnerImpl<Bindings> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModuleStateInner").finish_non_exhaustive()
    }
}

#[cfg(feature = "wit")]
impl<Bindings: BindingsBound> ModuleStateInnerImpl<Bindings> {
    fn new(
        component_bytecode: &[u8],
        stdout_output: Box<dyn Fn(&World, &str) + Sync + Send>,
        stderr_output: Box<dyn Fn(&World, &str) + Sync + Send>,
        bindings: Bindings,
    ) -> anyhow::Result<Self> {
        let engine = &*crate::WASMTIME_ENGINE;

        let (stdout_output, stdout_consumer) = WasiOutputStream::make(stdout_output);
        let (stderr_output, stderr_consumer) = WasiOutputStream::make(stderr_output);
        let mut table = wasi_preview2::Table::new();
        let wasi = wasi_preview2::WasiCtxBuilder::new()
            .set_stdout(stdout_output)
            .set_stderr(stderr_output)
            .build(&mut table)?;
        let mut store = wasmtime::Store::new(
            engine,
            WasmtimeContext {
                wasi,
                bindings,
                table,
            },
        );

        let mut linker = wasmtime::component::Linker::<WasmtimeContext<Bindings>>::new(engine);
        wasi_preview2::wasi::command::add_to_linker(&mut linker)?;
        shared::wit::Bindings::add_to_linker(&mut linker, |x| &mut x.bindings)?;

        let component = wasmtime::component::Component::from_binary(engine, component_bytecode)?;

        let (guest_bindings, guest_instance) = pollster::block_on(async {
            let (guest_bindings, guest_instance) =
                shared::wit::Bindings::instantiate_async(&mut store, &component, &linker).await?;

            // // Initialise the runtime.
            guest_bindings
                .ambient_bindings_guest()
                .call_init(&mut store)
                .await?;

            anyhow::Ok((guest_bindings, guest_instance))
        })?;

        Ok(Self {
            store,
            guest_bindings,
            _guest_instance: guest_instance,

            stdout_consumer,
            stderr_consumer,
        })
    }
}

#[cfg(feature = "wit")]
impl<Bindings: BindingsBound> ModuleStateBehavior for ModuleStateInnerImpl<Bindings> {
    fn run(
        &mut self,
        world: &mut World,
        message_source: &Source,
        message_name: &str,
        message_data: &[u8],
    ) -> anyhow::Result<()> {
        self.store.data_mut().bindings.set_world(world);

        let guest = &self.guest_bindings.ambient_bindings_guest();
        let result = pollster::block_on(guest.call_exec(
            &mut self.store,
            &match message_source {
                Source::Runtime => shared::wit::guest::Source::Runtime,
                Source::Server => shared::wit::guest::Source::Server,
                Source::Client(user_id) => shared::wit::guest::Source::Client(user_id.clone()),
                Source::Local(module) => shared::wit::guest::Source::Local(module.into_bindgen()),
            },
            message_name,
            message_data,
        ));

        self.store.data_mut().bindings.clear_world();

        self.stdout_consumer.process_incoming(world);
        self.stderr_consumer.process_incoming(world);

        result
    }

    fn drain_spawned_entities(&mut self) -> HashSet<EntityId> {
        std::mem::take(&mut self.store.data_mut().bindings.base_mut().spawned_entities)
    }

    fn listen_to_message(&mut self, event_name: String) {
        self.store
            .data_mut()
            .bindings
            .base_mut()
            .subscribed_messages
            .insert(event_name);
    }

    fn supports_message(&self, event_name: &str) -> bool {
        self.store
            .data()
            .bindings
            .base()
            .subscribed_messages
            .contains(event_name)
    }
}

struct WasiOutputStream(flume::Sender<String>);
impl WasiOutputStream {
    fn make(
        outputter: Box<dyn Fn(&World, &str) + Sync + Send>,
    ) -> (Self, WasiOutputStreamConsumer) {
        let (tx, rx) = flume::unbounded();
        (
            Self(tx),
            WasiOutputStreamConsumer {
                rx,
                outputter,
                buffer: String::new(),
            },
        )
    }
}

#[async_trait::async_trait]
#[cfg(feature = "native")]
impl wasi_preview2::OutputStream for WasiOutputStream {
    fn as_any(&self) -> &dyn Any {
        self
    }

    async fn writable(&self) -> Result<(), anyhow::Error> {
        Ok(())
    }

    async fn write(&mut self, buf: &[u8]) -> Result<u64, anyhow::Error> {
        let msg = std::str::from_utf8(buf)?;
        self.0.send(msg.to_string())?;

        Ok(buf.len().try_into()?)
    }
}

struct WasiOutputStreamConsumer {
    rx: flume::Receiver<String>,
    outputter: Box<dyn Fn(&World, &str) + Sync + Send>,
    buffer: String,
}

impl WasiOutputStreamConsumer {
    fn process_incoming(&mut self, world: &World) {
        for msg in self.rx.drain() {
            self.buffer += &msg;
        }

        if self.buffer.contains('\n') {
            for line in self.buffer.lines() {
                (self.outputter)(world, line);
            }
            self.buffer.clear();
        }
    }
}
