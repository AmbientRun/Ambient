use std::{io, sync::Arc};

use anyhow::Context;
use async_trait::async_trait;
use elements_ecs::{EntityData, World};
use parking_lot::Mutex;
use wasi_common::{
    file::{FdFlags, FileType},
    WasiCtx,
};
use wasmtime_wasi::WasiFile;

use super::ScriptContext;

pub struct WorldRef(pub *mut World);
impl WorldRef {
    pub const fn new() -> Self {
        WorldRef(std::ptr::null_mut())
    }
}
unsafe impl Send for WorldRef {}
unsafe impl Sync for WorldRef {}
impl AsRef<World> for WorldRef {
    fn as_ref(&self) -> &World {
        unsafe { self.0.as_ref().unwrap() }
    }
}
impl AsMut<World> for WorldRef {
    fn as_mut(&mut self) -> &mut World {
        unsafe { self.0.as_mut().unwrap() }
    }
}

// TODO(mithun): come up with a more optimal way to do this that doesn't
// implicitly require unsafe and mutex locking
struct WasiOutputFile(
    Box<dyn Fn(&World, &str) + Sync + Send>,
    Arc<Mutex<WorldRef>>,
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
        bufs: &[io::IoSlice<'a>],
    ) -> Result<u64, wasmtime_wasi::Error> {
        let mut count = 0;
        for buf in bufs {
            if let Ok(text) = std::str::from_utf8(buf) {
                self.0(self.1.lock().as_ref(), text);
                count += text.len();
            }
        }
        Ok(count as u64)
    }
}

pub trait WasmContext {
    fn wasi(&mut self) -> &mut wasmtime_wasi::WasiCtx;
    fn set_world(&mut self, world: &mut World);
}

pub trait GuestExports<WasmContext>
where
    Self: Sized,
{
    fn create(
        engine: &wasmtime::Engine,
        store: &mut wasmtime::Store<WasmContext>,
        linker: &mut wasmtime::Linker<WasmContext>,
        bytecode: &[u8],
    ) -> anyhow::Result<(Self, wasmtime::Instance)>;

    fn initialize(&self, store: &mut wasmtime::Store<WasmContext>) -> anyhow::Result<()>;

    fn run(
        &self,
        store: &mut wasmtime::Store<WasmContext>,
        event_name: &str,
        components: &EntityData,
        time: f32,
        frametime: f32,
    ) -> anyhow::Result<()>;
}

pub struct WasmState<Context: WasmContext, Exports: GuestExports<Context>> {
    _engine: wasmtime::Engine,
    store: Arc<Mutex<wasmtime::Store<Context>>>,
    world_ref: Arc<Mutex<WorldRef>>,

    guest_exports: Arc<Exports>,
    _guest_instance: wasmtime::Instance,
}
impl<Context: WasmContext, Exports: GuestExports<Context>> Clone for WasmState<Context, Exports> {
    fn clone(&self) -> Self {
        Self {
            _engine: self._engine.clone(),
            store: self.store.clone(),
            world_ref: self.world_ref.clone(),
            guest_exports: self.guest_exports.clone(),
            _guest_instance: self._guest_instance,
        }
    }
}
impl<Context: WasmContext, Exports: GuestExports<Context>> WasmState<Context, Exports> {
    pub fn new(
        bytecode: &[u8],
        stdout_output: Box<dyn Fn(&World, &str) + Sync + Send>,
        stderr_output: Box<dyn Fn(&World, &str) + Sync + Send>,
        make_wasm_context: impl Fn(WasiCtx) -> Context,
        interface_version: u32,
    ) -> anyhow::Result<Self> {
        let engine = wasmtime::Engine::default();
        let world_ref = Arc::new(Mutex::new(WorldRef::new()));
        let mut store = wasmtime::Store::new(
            &engine,
            make_wasm_context(
                wasmtime_wasi::sync::WasiCtxBuilder::new()
                    .stdout(Box::new(WasiOutputFile(stdout_output, world_ref.clone())))
                    .stderr(Box::new(WasiOutputFile(stderr_output, world_ref.clone())))
                    .build(),
            ),
        );

        let (guest_exports, guest_instance) = {
            let mut linker: wasmtime::Linker<Context> = wasmtime::Linker::new(&engine);
            wasmtime_wasi::add_to_linker(&mut linker, |cx| cx.wasi())?;

            Exports::create(&engine, &mut store, &mut linker, bytecode)?
        };

        // Initialise the runtime.
        guest_exports.initialize(&mut store)?;
        // Call the script's main function.
        guest_instance
            .get_func(&mut store, "call_main")
            .context("not a func")?
            .typed::<(u32,), (), _>(&store)?
            .call(&mut store, (interface_version,))?;

        Ok(WasmState {
            _engine: engine,
            store: Arc::new(Mutex::new(store)),
            world_ref,

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

        let mut store = self.store.lock();
        store.data_mut().set_world(world);
        self.world_ref.lock().0 = world;
        self.guest_exports
            .run(&mut *store, event_name, event_data, *time, *frametime)
    }
}
