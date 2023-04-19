use std::{collections::HashMap, path::{PathBuf, Component}, sync::Arc, borrow::Cow};

use ambient_ecs::{EntityId, SystemGroup, World};
use ambient_project::Identifier;
use ambient_std::asset_url::AbsAssetUrl;
pub use ambient_wasm::server::{on_forking_systems, on_shutdown_systems};
use ambient_wasm::shared::{
    client_bytecode_from_url, get_module_name, module_bytecode, remote_paired_id, spawn_module, MessageType, ModuleBytecode,
};
use anyhow::Context;

pub fn systems() -> SystemGroup {
    ambient_wasm::server::systems()
}

pub fn initialize(world: &mut World, project_path: PathBuf, manifest: &ambient_project::Manifest) -> anyhow::Result<()> {
    let messenger = Arc::new(|world: &World, id: EntityId, type_: MessageType, message: &str| {
        let name = get_module_name(world, id);
        let (prefix, level) = match type_ {
            MessageType::Info => ("info", log::Level::Info),
            MessageType::Warn => ("warn", log::Level::Warn),
            MessageType::Error => ("error", log::Level::Error),
            MessageType::Stdout => ("stdout", log::Level::Info),
            MessageType::Stderr => ("stderr", log::Level::Info),
        };

        log::log!(level, "[{name}] {prefix}: {}", message.strip_suffix('\n').unwrap_or(message));
    });

    ambient_wasm::server::initialize(world, messenger)?;

    let build_dir = project_path.join("build");

    let mut modules_to_entity_ids = HashMap::new();
    for target in ["client", "server"] {
        let wasm_component_paths: Vec<PathBuf> = std::fs::read_dir(build_dir.join(target))
            .ok()
            .map(|rd| rd.filter_map(Result::ok).map(|p| p.path()).filter(|p| p.extension().unwrap_or_default() == "wasm").collect())
            .unwrap_or_default();

        let is_sole_module = wasm_component_paths.len() == 1;
        for path in wasm_component_paths {
            let name =
                Identifier::new(&*path.file_stem().context("no file stem for {path:?}")?.to_string_lossy()).map_err(anyhow::Error::msg)?;

            let description = manifest.project.description.clone().unwrap_or_default();
            let description = if is_sole_module { description } else { format!("{description} ({name})") };

            let id = spawn_module(world, &name, description, true);
            modules_to_entity_ids.insert(
                (
                    target,
                    // Support `client_module`, `module_client` and `module`
                    name.as_ref()
                        .strip_prefix(target)
                        .or_else(|| name.as_ref().strip_suffix(target))
                        .unwrap_or(name.as_ref())
                        .trim_matches('_')
                        .to_string(),
                ),
                id,
            );

            if target == "client" {
                let relative_path = path.strip_prefix(&build_dir)?;
                let asset_key: String = itertools::Itertools::intersperse(
                    relative_path
                        .components()
                        .map(|c| match c {
                            Component::Normal(c) => c.to_string_lossy(),
                            _ => unreachable!(),
                        }),
                    Cow::Borrowed("/"),
                ).collect();
                let bytecode_url = AbsAssetUrl::from_asset_key(asset_key).to_string();
                world.add_component(id, client_bytecode_from_url(), bytecode_url)?;
            } else {
                let bytecode = std::fs::read(path)?;
                world.add_component(id, module_bytecode(), ModuleBytecode(bytecode))?;
            }
        }
    }

    for ((target, name), id) in modules_to_entity_ids.iter() {
        let corresponding = match *target {
            "client" => "server",
            "server" => "client",
            _ => unreachable!(),
        };
        if let Some(other_id) = modules_to_entity_ids.get(&(corresponding, name.clone())) {
            world.add_component(*id, remote_paired_id(), *other_id)?;
        }
    }

    Ok(())
}
