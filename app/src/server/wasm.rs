use std::{collections::HashMap, fmt::Display, path::PathBuf, sync::Arc};

use ambient_ecs::{EntityId, SystemGroup, World};
use ambient_native_std::asset_cache::AssetCache;
use ambient_package_semantic_native::{WasmSpawnRequest, WasmSpawnResponse};
pub use ambient_wasm::server::{on_forking_systems, on_shutdown_systems};
use ambient_wasm::shared::{module_name, remote_paired_id, spawn_module, MessageType};
use anyhow::Context;

pub fn systems() -> SystemGroup {
    ambient_wasm::server::systems()
}

pub async fn initialize(
    world: &mut World,
    assets: &AssetCache,
    data_path: PathBuf,
) -> anyhow::Result<()> {
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

    ambient_wasm::server::initialize(world, assets, data_path, messenger)?;

    Ok(())
}

/// `enabled` is passed here as we need knowledge of the other packages to determine if this package should be enabled or not
pub fn spawn_package(
    world: &mut World,
    request: WasmSpawnRequest,
) -> anyhow::Result<WasmSpawnResponse> {
    let WasmSpawnRequest {
        client_modules,
        server_modules,
    } = request;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum Side {
        Client,
        Server,
    }
    impl Side {
        fn as_str(&self) -> &'static str {
            match self {
                Side::Client => "client",
                Side::Server => "server",
            }
        }
        fn corresponding(&self) -> Self {
            match self {
                Side::Client => Side::Server,
                Side::Server => Side::Client,
            }
        }
    }
    impl Display for Side {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.as_str())
        }
    }

    // Spawn all of the modules for each side, collecting them as we go
    let mut modules_to_entity_ids = HashMap::new();
    for (target, modules) in [
        (Side::Client, client_modules),
        (Side::Server, server_modules),
    ] {
        for (url, enabled) in modules {
            let name = url
                .file_stem()
                .context("no file stem for {url}")?
                .to_owned();

            let id = spawn_module(world, url, enabled, target == Side::Server);
            modules_to_entity_ids.insert(
                (
                    target,
                    // Support `client_module`, `module_client` and `module`
                    name.strip_prefix(target.as_str())
                        .or_else(|| name.strip_suffix(target.as_str()))
                        .unwrap_or(name.as_ref())
                        .trim_matches('_')
                        .to_string(),
                ),
                id,
            );
        }
    }

    // Associate a module with its paired module
    // TODO: make this send to the package instead of the module so that we get routing for free
    for ((target, name), id) in modules_to_entity_ids.iter() {
        if let Some(other_id) = modules_to_entity_ids.get(&(target.corresponding(), name.clone())) {
            world.add_component(*id, remote_paired_id(), *other_id)?;
        }
    }

    // Collect the modules that were spawned
    let mut client_modules = vec![];
    let mut server_modules = vec![];
    for ((target, _), id) in modules_to_entity_ids {
        let modules = match target {
            Side::Client => &mut client_modules,
            Side::Server => &mut server_modules,
        };
        modules.push(id);
    }

    Ok(WasmSpawnResponse {
        client_modules,
        server_modules,
    })
}
