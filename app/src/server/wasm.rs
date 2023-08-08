use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use ambient_ecs::{EntityId, SystemGroup, World};
use ambient_native_std::{asset_url::AbsAssetUrl, Cb};
use ambient_project_semantic::{ItemId, Scope};
pub use ambient_wasm::server::{on_forking_systems, on_shutdown_systems};
use ambient_wasm::shared::{module_name, remote_paired_id, spawn_module, MessageType};
use anyhow::Context;

pub fn systems() -> SystemGroup {
    ambient_wasm::server::systems()
}

pub async fn initialize(
    world: &mut World,
    data_path: PathBuf,
    build_project: Option<Cb<dyn Fn(&mut World) + Send + Sync>>,
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

    ambient_wasm::server::initialize(world, data_path, messenger, build_project)?;

    Ok(())
}

pub fn instantiate_ember(world: &mut World, ember_id: ItemId<Scope>) -> anyhow::Result<()> {
    let semantic = world.resource(super::semantic()).clone();
    let semantic = semantic.lock().unwrap();

    let mut modules_to_entity_ids = HashMap::new();
    for target in ["client", "server"] {
        let scope = semantic.items.get(ember_id)?;
        let build_metadata = scope
            .build_metadata
            .as_ref()
            .context("no build metadata in ember")?;
        let wasm_component_paths: &[String] = build_metadata.component_paths(target);

        for path in wasm_component_paths {
            let path = Path::new(scope.original_id.as_str()).join(path);
            let name = path
                .file_stem()
                .context("no file stem for {path:?}")?
                .to_string_lossy();

            let bytecode_url = AbsAssetUrl::from_asset_key(path.to_string_lossy())?;
            let id = spawn_module(world, bytecode_url, true, target == "server");
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
