use std::{path::PathBuf, sync::Arc};

use ambient_ecs::{components, EntityId, SystemGroup, World};
use ambient_network::server::{ForkingEvent, ShutdownEvent};
use ambient_wasm::shared::{get_module_name, module_bytecode, spawn_module, MessageType, ModuleBytecode};

components!("wasm::server", {});

pub fn init_all_components() {
    init_components();
}

pub fn systems() -> SystemGroup {
    ambient_wasm::shared::systems()
}

pub fn on_forking_systems() -> SystemGroup<ForkingEvent> {
    ambient_wasm::shared::on_forking_systems()
}

pub fn on_shutdown_systems() -> SystemGroup<ShutdownEvent> {
    ambient_wasm::shared::on_shutdown_systems()
}

pub async fn initialize(world: &mut World, project_path: PathBuf, manifest: &ambient_project::Manifest) -> anyhow::Result<()> {
    let messenger = Arc::new(|world: &World, id: EntityId, type_: MessageType, message: &str| {
        let name = get_module_name(world, id);
        let (prefix, level) = match type_ {
            MessageType::Info => ("info", log::Level::Info),
            MessageType::Error => ("error", log::Level::Error),
            MessageType::Stdout => ("stdout", log::Level::Info),
            MessageType::Stderr => ("stderr", log::Level::Info),
        };

        log::log!(level, "[{name}] {prefix}: {}", message.strip_suffix('\n').unwrap_or(message));
    });

    ambient_wasm::server::initialize(world, messenger)?;

    let main_wasm_path = project_path.join("build").join(format!("{}.wasm", manifest.project.id));
    if main_wasm_path.exists() {
        let bytecode = std::fs::read(main_wasm_path)?;

        let id = spawn_module(world, &manifest.project.id, manifest.project.description.clone().unwrap_or_default(), true)?;
        world.add_component(id, module_bytecode(), ModuleBytecode(bytecode))?;
    }

    Ok(())
}
