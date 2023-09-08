use std::{fmt::Display, path::PathBuf, sync::Arc};

use ambient_ecs::{Entity, EntityId, SystemGroup, World};
use ambient_native_std::asset_cache::AssetCache;
use ambient_package_semantic_native::{WasmSpawnRequest, WasmSpawnResponse};
pub use ambient_wasm::server::{on_forking_systems, on_shutdown_systems};
use ambient_wasm::shared::{
    bytecode_from_url, is_module, is_module_on_server, module_enabled, module_name, package_ref,
    MessageType,
};

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
        package_id,
        client_modules: client_request,
        server_modules: server_request,
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
    }
    impl Display for Side {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.as_str())
        }
    }

    // Spawn all of the modules for each side, collecting them as we go
    let mut client_modules = vec![];
    let mut server_modules = vec![];
    for (target, modules) in [
        (Side::Client, client_request),
        (Side::Server, server_request),
    ] {
        for (url, enabled) in modules {
            let entity = Entity::new()
                .with(self::is_module(), ())
                .with(self::bytecode_from_url(), url.to_string())
                .with(self::module_enabled(), enabled)
                .with(self::package_ref(), package_id);

            let is_server = target == Side::Server;
            let entity = if is_server {
                entity.with(is_module_on_server(), ())
            } else {
                entity
            };

            let id = entity.spawn(world);
            if is_server {
                server_modules.push(id);
            } else {
                client_modules.push(id);
            }
        }
    }

    Ok(WasmSpawnResponse {
        client_modules,
        server_modules,
    })
}
