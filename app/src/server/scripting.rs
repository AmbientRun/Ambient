use std::{path::PathBuf, sync::Arc};

use elements_ecs::{components, EntityId, SystemGroup, World};
use elements_network::server::{ForkingEvent, ShutdownEvent};
use elements_scripting_host::{
    server::bindings::{Bindings as ElementsBindings, WasmServerContext}, shared::{
        host_guest_state::BaseHostGuestState, script_module_bytecode, spawn_script, util::get_module_name, MessageType, ScriptModuleBytecode, ScriptModuleState
    }, Linker, WasiCtx
};
use parking_lot::RwLock;

pub type ScriptModuleServerState = ScriptModuleState<ElementsBindings, WasmServerContext, BaseHostGuestState>;

components!("scripting::server", {
    // component
    script_module_state: ScriptModuleServerState,
    // resource
    make_wasm_context: Arc<dyn Fn(WasiCtx, Arc<RwLock<BaseHostGuestState>>) -> WasmServerContext + Send + Sync>,
    add_to_linker: Arc<dyn Fn(&mut Linker<WasmServerContext>) -> anyhow::Result<()> + Send + Sync>,
});

pub fn init_all_components() {
    init_components();
}

pub fn systems() -> SystemGroup {
    elements_scripting_host::server::systems(script_module_state(), make_wasm_context(), add_to_linker())
}

pub fn on_forking_systems() -> SystemGroup<ForkingEvent> {
    elements_scripting_host::server::on_forking_systems(script_module_state(), make_wasm_context(), add_to_linker())
}

pub fn on_shutdown_systems() -> SystemGroup<ShutdownEvent> {
    elements_scripting_host::server::on_shutdown_systems(script_module_state())
}

pub async fn initialize(world: &mut World, project_path: PathBuf, manifest: &elements_project::Manifest) -> anyhow::Result<()> {
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

    elements_scripting_host::server::initialize(
        world,
        messenger,
        (make_wasm_context(), Arc::new(|ctx, state| WasmServerContext::new(ctx, state))),
        (add_to_linker(), Arc::new(|linker| WasmServerContext::link(linker, |c| c))),
    )
    .await?;

    let main_wasm_path = project_path.join("target").join(format!("{}.wasm", manifest.project.id));
    if main_wasm_path.exists() {
        let bytecode = std::fs::read(main_wasm_path)?;

        let id = spawn_script(world, &manifest.project.id, manifest.project.description.clone().unwrap_or_default(), true)?;
        world.add_component(id, script_module_bytecode(), ScriptModuleBytecode(bytecode))?;
    }

    Ok(())
}
