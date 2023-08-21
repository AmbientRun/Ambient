use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use ambient_ecs::{EntityId, SystemGroup, World};
use ambient_package_semantic::{ItemId, Package};
pub use ambient_wasm::server::{on_forking_systems, on_shutdown_systems};
use ambient_wasm::shared::{module_name, remote_paired_id, spawn_module, MessageType};
use anyhow::Context;

pub fn systems() -> SystemGroup {
    ambient_wasm::server::systems()
}

pub async fn initialize(world: &mut World, data_path: PathBuf) -> anyhow::Result<()> {
    let messenger = Arc::new(
        |world: &World, id: EntityId, type_: MessageType, message: &str| {
            let name = world.get_cloned(id, module_name()).unwrap_or_default();
            let (prefix, level) = match type_ {
                MessageType::Info => ("info", log::Level::Info),
                MessageType::Warn => ("warn", log::Level::Warn),
                MessageType::Error => ("error", log::Level::Error),
                MessageType::Stdout => ("stdout", log::Level::Info),
                MessageType::Stderr => ("stderr", log::Level::Info),
            };

            log::log!(
                level,
                "[{name}] {prefix}: {}",
                message.strip_suffix('\n').unwrap_or(message)
            );
        },
    );

    ambient_wasm::server::initialize(world, data_path, messenger)?;

    Ok(())
}

/// `enabled` is passed here as we need knowledge of the other packages to determine if this package should be enabled or not
pub fn instantiate_package(
    world: &mut World,
    package_id: ItemId<Package>,
    enabled: bool,
) -> anyhow::Result<()> {
    let mut modules_to_entity_ids = HashMap::new();
    for target in ["client", "server"] {
        let (package_name, package_enabled, build_metadata) = {
            let semantic = ambient_package_semantic_native::world_semantic(world);
            let semantic = semantic.lock().unwrap();
            let scope = semantic.items.get(package_id);

            (
                scope.data.id.to_string(),
                enabled,
                scope
                    .build_metadata
                    .as_ref()
                    .context("no build metadata in package")?
                    .clone(),
            )
        };
        let wasm_component_paths: &[String] = build_metadata.component_paths(target);

        for path in wasm_component_paths {
            let path = Path::new(path);
            let name = path
                .file_stem()
                .context("no file stem for {path:?}")?
                .to_string_lossy();

            let bytecode_url =
                ambient_package_semantic_native::file_path(world, &package_name, path)?;
            let id = spawn_module(world, bytecode_url, package_enabled, target == "server");
            modules_to_entity_ids.insert(
                (
                    target,
                    // Support `client_module`, `module_client` and `module`
                    name.strip_prefix(target)
                        .or_else(|| name.strip_suffix(target))
                        .unwrap_or(name.as_ref())
                        .trim_matches('_')
                        .to_string(),
                ),
                id,
            );
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
