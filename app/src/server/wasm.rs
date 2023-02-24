use std::{path::PathBuf, sync::Arc};

use ambient_ecs::{components, EntityId, Resource, SystemGroup, World};
use ambient_network::server::{ForkingEvent, ShutdownEvent};
use ambient_wasm::{
    server::bindings::{Bindings as ElementsBindings, WasmServerContext},
    shared::{
        get_module_name, host_guest_state::BaseHostGuestState, module_bytecode, spawn_module, MessageType, ModuleBytecode, ModuleState,
    },
    Linker, WasiCtx,
};
use parking_lot::RwLock;

pub type ModuleServerState = ModuleState<ElementsBindings, WasmServerContext, BaseHostGuestState>;

components!("wasm::server", {
    module_state: ModuleServerState,

    @[Resource]
    make_wasm_context: Arc<dyn Fn(WasiCtx, Arc<RwLock<BaseHostGuestState>>) -> WasmServerContext + Send + Sync>,
    @[Resource]
    add_to_linker: Arc<dyn Fn(&mut Linker<WasmServerContext>) -> anyhow::Result<()> + Send + Sync>,
});

pub fn init_all_components() {
    init_components();
}

pub fn systems() -> SystemGroup {
    ambient_wasm::server::systems(module_state(), make_wasm_context(), add_to_linker())
}

pub fn on_forking_systems() -> SystemGroup<ForkingEvent> {
    ambient_wasm::server::on_forking_systems(module_state(), make_wasm_context(), add_to_linker())
}

pub fn on_shutdown_systems() -> SystemGroup<ShutdownEvent> {
    ambient_wasm::server::on_shutdown_systems(module_state())
}

pub async fn initialize(world: &mut World, project_path: PathBuf, manifest: &ambient_project::Manifest) -> anyhow::Result<()> {
    let messenger = Arc::new(|world: &World, id: EntityId, type_: MessageType, message: &str| {
        let name = get_module_name(world, id);
        let (prefix, level) = match type_ {
            MessageType::Info => ("info", log::Level::Info),
            MessageType::Error => ("error", log::Level::Error),
            MessageType::Stdout => ("stdout", log::Level::Info),
            MessageType::Stderr => ("stderr", log::Level::Warn),
        };

        log::log!(level, "[{name}] {prefix}: {}", message.strip_suffix('\n').unwrap_or(message));
    });

    ambient_wasm::server::initialize(
        world,
        messenger,
        (make_wasm_context(), Arc::new(|ctx, state| WasmServerContext::new(ctx, state))),
        (add_to_linker(), Arc::new(|linker| WasmServerContext::link(linker, |c| c))),
    )
    .await?;

    let main_wasm_path = project_path.join("build").join(format!("{}.wasm", manifest.project.id));
    if main_wasm_path.exists() {
        let bytecode = std::fs::read(main_wasm_path)?;

        let id = spawn_module(world, &manifest.project.id, manifest.project.description.clone().unwrap_or_default(), true)?;
        world.add_component(id, module_bytecode(), ModuleBytecode(bytecode))?;
    }

    Ok(())
}
