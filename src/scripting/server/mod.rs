use std::{collections::HashMap, path::PathBuf, sync::Arc};

use elements_ecs::{components, EntityId, SystemGroup, World};
use elements_network::server::{ForkingEvent, ShutdownEvent};
pub use elements_scripting_host::server::docs_path;
use elements_scripting_host::{
    server::{bindings::Bindings, rustc::InstallDirs, spawn_script_module, HostState, MessageType},
    shared::{
        get_module_name,
        interface::{
            get_scripting_interfaces,
            guest::{Guest, GuestData},
        },
        wasm::WasmContext,
        BaseHostGuestState, File, ScriptModuleState,
    },
};
use parking_lot::Mutex;

use crate::server::project_path;

pub type HostServerState =
    HostState<Bindings, WasmServerContext, Guest<WasmServerContext>, BaseHostGuestState>;

pub type ScriptModuleServerState =
    ScriptModuleState<Bindings, WasmServerContext, Guest<WasmServerContext>, BaseHostGuestState>;

components!("scripting::server", {
    // resource
    host_state: Arc<HostServerState>,
    // component
    script_module_state: ScriptModuleServerState,
});

pub struct WasmServerContext {
    pub wasi: wasmtime_wasi::WasiCtx,
    pub elements_bindings: Bindings,
    pub guest_data: GuestData,
}
impl WasmServerContext {
    pub fn new(wasi: wasmtime_wasi::WasiCtx, shared_state: Arc<Mutex<BaseHostGuestState>>) -> Self {
        Self {
            wasi,
            elements_bindings: Bindings::new(shared_state.clone()),
            guest_data: GuestData::default(),
        }
    }
}
impl WasmContext<Bindings> for WasmServerContext {
    fn wasi(&mut self) -> &mut wasmtime_wasi::WasiCtx {
        &mut self.wasi
    }

    fn set_world(&mut self, world: &mut elements_ecs::World) {
        self.elements_bindings.set_world(world);
    }

    fn bindings_implementation(&mut self) -> &mut Bindings {
        &mut self.elements_bindings
    }

    fn guest_data(&mut self) -> &mut elements_scripting_host::shared::interface::guest::GuestData {
        &mut self.guest_data
    }
}

pub fn init_all_components() {
    elements_scripting_host::server::init_components();
    init_components();
}

pub fn systems() -> SystemGroup {
    elements_scripting_host::server::systems(host_state(), false)
}

pub fn on_forking_systems() -> SystemGroup<ForkingEvent> {
    elements_scripting_host::server::on_forking_systems(host_state())
}

pub fn on_shutdown_systems() -> SystemGroup<ShutdownEvent> {
    elements_scripting_host::server::on_shutdown_systems(host_state())
}

pub async fn initialize(world: &mut World) -> anyhow::Result<()> {
    let rust_path = elements_std::path::normalize(&std::env::current_dir()?.join("rust"));

    let messenger = Arc::new(
        |world: &World, id: EntityId, type_: MessageType, message: &str| {
            let name = get_module_name(world, id);
            let (prefix, level) = match type_ {
                MessageType::Info => ("info", log::Level::Info),
                MessageType::Error => ("error", log::Level::Error),
                MessageType::Stdout => ("stdout", log::Level::Info),
                MessageType::Stderr => ("stderr", log::Level::Warn),
            };

            log::log!(
                level,
                "[{name}] {prefix}: {}",
                message.strip_suffix('\n').unwrap_or(message)
            );
        },
    );

    let project_path = world.resource(project_path()).clone();
    let new_host_state = HostServerState {
        messenger,
        scripting_interfaces: get_scripting_interfaces(),

        rust_path: rust_path.clone(),
        install_dirs: InstallDirs {
            rustup_path: rust_path.join("rustup"),
            cargo_path: rust_path.join("cargo"),
        },
        scripting_interface_root_path: project_path.join("interfaces"),
        templates_path: rust_path.join("templates"),
        workspace_path: project_path.clone(),
        scripts_path: project_path.join("scripts"),

        server_state_component: script_module_state(),
        make_wasm_context: Arc::new(WasmServerContext::new),
        add_to_linker: Arc::new(|_linker| Ok(())),

        _bindings: Default::default(),
    };
    new_host_state
        .initialize(world, "elements_scripting_interface")
        .await?;
    world.add_resource(host_state(), Arc::new(new_host_state));

    let scripts_path = project_path.join("scripts");
    if scripts_path.exists() {
        for path in std::fs::read_dir(scripts_path)?
            .filter_map(Result::ok)
            .map(|de| de.path())
            .filter(|p| p.is_dir())
            .filter(|p| p.join("Cargo.toml").exists())
        {
            if let Some(file_name) = path.file_name() {
                let name = file_name.to_string_lossy();

                let files: HashMap<PathBuf, File> = walkdir::WalkDir::new(&path)
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter(|de| de.path().is_file())
                    .map(|de| {
                        Ok((
                            de.path().strip_prefix(&path)?.to_path_buf(),
                            File::new_at_now(std::fs::read_to_string(de.path())?),
                        ))
                    })
                    .collect::<anyhow::Result<_>>()?;

                spawn_script_module(
                    world,
                    name.as_ref(),
                    String::new(),
                    true,
                    files,
                    Default::default(),
                    Default::default(),
                )?;
            }
        }
    }

    Ok(())
}
