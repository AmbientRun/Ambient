use std::{path::PathBuf, sync::Arc};

use ambient_ecs::{EntityId, SystemGroup, World};
use ambient_project::Identifier;
pub use ambient_wasm::server::{on_forking_systems, on_shutdown_systems};
use ambient_wasm::shared::{client_module_bytecode, get_module_name, module_bytecode, spawn_module, MessageType, ModuleBytecode};
use anyhow::Context;

pub fn systems() -> SystemGroup {
    ambient_wasm::server::systems()
}

pub fn initialize(world: &mut World, project_path: PathBuf, manifest: &ambient_project::Manifest) -> anyhow::Result<()> {
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

    for target in ["client", "server"] {
        let wasm_component_paths: Vec<PathBuf> = std::fs::read_dir(project_path.join("build").join(target))
            .ok()
            .map(|rd| rd.filter_map(Result::ok).map(|p| p.path()).filter(|p| p.extension().unwrap_or_default() == "wasm").collect())
            .unwrap_or_default();

        let is_sole_module = wasm_component_paths.len() == 1;
        for path in wasm_component_paths {
            let filename_identifier =
                Identifier::new(&*path.file_stem().context("no file stem for {path:?}")?.to_string_lossy()).map_err(anyhow::Error::msg)?;

            let bytecode = std::fs::read(path)?;
            let name = if is_sole_module {
                manifest.project.id.clone()
            } else {
                Identifier::new(format!("{}_{}", manifest.project.id, filename_identifier)).map_err(anyhow::Error::msg)?
            };

            let description = manifest.project.description.clone().unwrap_or_default();
            let description = if is_sole_module { description } else { format!("{description} ({filename_identifier})") };

            let id = spawn_module(world, &name, description, true)?;

            let component = if target == "client" { client_module_bytecode() } else { module_bytecode() };
            world.add_component(id, component, ModuleBytecode(bytecode))?;
        }
    }

    Ok(())
}
