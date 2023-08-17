use crate::shared;

use super::{bindings::BindingsBound, conversion::IntoBindgen};
use super::{engine::EngineKey, Source};
use ambient_ecs::{EntityId, World};
use ambient_native_std::asset_cache::{AssetCache, SyncAssetKeyExt};
use ambient_sys::task::PlatformBoxFuture;
use anyhow::Context;
use data_encoding::BASE64;
use flume::{SendError, TrySendError};
use futures::future::BoxFuture;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::io;
use std::{any::Any, collections::HashSet, sync::Arc};
use wasm_bridge::Store;

// use wasi_cap_std_sync::Dir;
// use wasmtime_wasi::preview2 as wasi_preview2;

#[cfg(not(target_os = "unknown"))]
use wasmtime_wasi::preview2::{DirPerms, FilePerms};

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

/// Binding and linking table generic over the host and guest bindings
struct BindingContext<Bindings: BindingsBound> {
    bindings: Bindings,
    wasi: WasiCtx,
    table: Table,
}
#[cfg(target_os = "unknown")]
use wasm_bridge_js::wasi::preview2;
#[cfg(not(target_os = "unknown"))]
use wasmtime_wasi::preview2;
impl<B: BindingsBound> preview2::WasiView for BindingContext<B> {
    fn table(&self) -> &preview2::Table {
        &self.table
    }

    fn table_mut(&mut self) -> &mut preview2::Table {
        &mut self.table
    }

    fn ctx(&self) -> &preview2::WasiCtx {
        &self.wasi
    }

    fn ctx_mut(&mut self) -> &mut preview2::WasiCtx {
        &mut self.wasi
    }
}

pub trait ModuleStateBehavior: Send + Sync {
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
    #[cfg(not(target_os = "unknown"))]
    /// Makes the `data` directory available during development
    pub preopened_dir: Option<wasi_cap_std_sync::Dir>,
}

#[derive(Clone)]
pub struct ModuleState {
    // Wrap the inner state to make it easily cloneable and to allow for erasing
    // the precise bindings in use
    inner: Arc<RwLock<dyn ModuleStateBehavior>>,
}

impl ModuleState {
    async fn new<Bindings: BindingsBound + 'static>(
        assets: &AssetCache,
        args: ModuleStateArgs<'_>,
        bindings: fn(EntityId) -> Bindings,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            inner: Arc::new(RwLock::new(
                InstanceState::new(assets, args, bindings).await?,
            )),
        })
    }

    pub fn create_state_maker<Bindings: BindingsBound + 'static>(
        assets: &AssetCache,
        bindings: fn(EntityId) -> Bindings,
    ) -> Arc<
        dyn Fn(ModuleStateArgs<'_>) -> PlatformBoxFuture<anyhow::Result<ModuleState>> + Send + Sync,
    > {
        let assets = assets.clone();
        Arc::new(move |args: ModuleStateArgs<'_>| {
            // Generic over send or not send, depending on the target platform.
            //
            // I know, it is hacky... but it works and is sound
            PlatformBoxFuture::new(Self::new(&assets, args, bindings))
        })
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

#[cfg(target_os = "unknown")]
use wasm_bridge_js::{
    component::{self, Instance},
    wasi::preview2::{wasi::command::add_to_linker, Table, WasiCtx, WasiCtxBuilder},
};

#[cfg(not(target_os = "unknown"))]
use wasmtime::{
    component::{self, Instance},
    wasi::preview2::Table,
    Store, WasiCtxBuilder,
};

/// Stores the execution context and store
struct InstanceState<Bindings: BindingsBound> {
    /// Stores the context and loaded instances
    store: Store<BindingContext<Bindings>>,

    guest_bindings: shared::wit::Bindings,
    _guest_instance: Instance,

    stdout_consumer: WasiOutputStreamConsumer,
    stderr_consumer: WasiOutputStreamConsumer,
}

impl<Bindings: BindingsBound> std::fmt::Debug for InstanceState<Bindings> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModuleStateInner").finish_non_exhaustive()
    }
}

impl<Bindings: BindingsBound> InstanceState<Bindings> {
    async fn new(
        assets: &AssetCache,
        args: ModuleStateArgs<'_>,
        bindings: fn(EntityId) -> Bindings,
    ) -> anyhow::Result<Self> {
        let bindings = bindings(args.id);

        let engine = EngineKey
            .get(assets)
            .map_err(|err| anyhow::anyhow!("{err:?}"))?;

        let (stdout_output, stdout_consumer) = WasiOutputStream::make(args.stdout_output);
        let (stderr_output, stderr_consumer) = WasiOutputStream::make(args.stderr_output);
        let mut table = Table::new();
        let wasi = WasiCtxBuilder::new()
            .set_stdout(stdout_output)
            .set_stderr(stderr_output);

        #[cfg(not(target_os = "unknown"))]
        let wasi = if let Some(dir) = args.preopened_dir {
            wasi.push_preopened_dir(dir, DirPerms::all(), FilePerms::all(), "/")
        } else {
            wasi
        };

        let wasi = wasi.build(&mut table)?;
        let mut store = Store::new(
            engine.inner(),
            BindingContext {
                wasi,
                bindings,
                table,
            },
        );

        // let mut store = wasmtime::Store::new(
        //     engine,
        //     ExecutionContext {
        //         wasi,
        //         bindings,
        //         table,
        //     },
        // );

        let mut linker =
            wasm_bridge::component::Linker::<BindingContext<Bindings>>::new(engine.inner());
        // let mut linker = wasmtime::component::Linker::<ExecutionContext<Bindings>>::new(engine);

        add_to_linker(&mut linker)?;

        shared::wit::Bindings::add_to_linker(&mut linker, |x| &mut x.bindings)?;

        let component = component::Component::new(engine.inner(), args.component_bytecode)?;

        let (guest_bindings, guest_instance) = async {
            let (guest_bindings, guest_instance) =
                shared::wit::Bindings::instantiate_async(&mut store, &component, &linker).await?;

            // Initialise the runtime.
            tracing::debug!(id=?args.id, "initialize runtime");
            guest_bindings
                .ambient_bindings_guest()
                .call_init(&mut store)
                .await?;

            anyhow::Ok((guest_bindings, guest_instance))
        }
        .await?;

        Ok(Self {
            store,
            guest_bindings,
            _guest_instance: guest_instance,

            stdout_consumer,
            stderr_consumer,
        })
    }
}

#[cfg(target_os = "unknown")]
mod miri_is_going_to_scream {

    use crate::shared::bindings::BindingsBound;

    use super::InstanceState;

    /// # Safety
    ///
    /// InstanceState contains an Instance, which is either a re-export of [`wasmtime::Instance`]`,
    /// or an implementation inside of [`wasm_bridge_js`]. On native, these are `Send` and `Sync`
    /// rightfully. However, on `wasm32` the `wasm_bridge_js` version uses `JsValue` which is
    /// inherently not `Send` nor `Sync`, as well `Rc`, `RefCell` etc because it is not thread-safe
    /// to begin with, so might as well use the `thread-local` specific containers.
    ///
    /// However, as much as I would like to adhere to soundness, this causes immense problems when
    /// we need to store an ignited ember inside the ECS world, as the world requires Send, and is
    /// in fact both sent and synced between threads on native due to tokio tasks and channels.
    ///
    /// On `wasm32-unknown-unknown` threading is more difficult, and as such is not used pervasively
    /// throughout programs such as async executors, and instead uses a single-threaded executor.
    /// Currently, we do not actually *use* the world on multiple threads, despite the APIs and
    /// trait objects requiring as such. This is because a `Box<dyn Trait + Send + Sync>` can not
    /// be parameterized over its bounds.
    ///
    /// This implementation is *only* sound iff the world or or any of its contained components,
    /// `Command`, `Entity`, `ComponentEntry` or callback et.al is *never* used in a multithreaded
    /// context.
    unsafe impl<Bindings> Send for InstanceState<Bindings> where Bindings: BindingsBound + Send + Sync {}
    unsafe impl<Bindings> Sync for InstanceState<Bindings> where Bindings: BindingsBound + Send + Sync {}
}

impl<Bindings: BindingsBound> ModuleStateBehavior for InstanceState<Bindings> {
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

#[cfg(target_os = "unknown")]
impl wasm_bridge_js::wasi::preview2::OutputStream for WasiOutputStream {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn writable(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn write(&mut self, buf: &[u8]) -> wasm_bridge::Result<u64> {
        let msg =
            std::str::from_utf8(buf).context("Non UTF-8 output is not currently supported")?;

        match self.0.try_send(msg.into()) {
            Ok(()) => Ok(msg.len() as u64),
            Err(TrySendError::Disconnected(_)) => {
                Err(io::Error::new(io::ErrorKind::BrokenPipe, "stdio disconnected").into())
            }
            Err(TrySendError::Full(_)) => {
                Err(io::Error::new(io::ErrorKind::WouldBlock, "stdio is full").into())
            }
        }
    }
}

#[cfg(not(target_os = "unknown"))]
#[async_trait::async_trait]
impl wasmtime_wasi::preview2::OutputStream for WasiOutputStream {
    fn as_any(&self) -> &dyn Any {
        self
    }

    async fn writable(&self) -> Result<(), anyhow::Error> {
        Ok(())
    }

    async fn write(&mut self, buf: &[u8]) -> wasm_bridge::Result<u64> {
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
