use std::{collections::HashMap, sync::Arc};

use ambient_ecs::{EntityId, SystemGroup, World};
use ambient_project::Identifier;
use ambient_std::asset_url::AbsAssetUrl;
pub use ambient_wasm::server::{on_forking_systems, on_shutdown_systems};
use ambient_wasm::shared::{
    bytecode_from_url, get_module_name, remote_paired_id, spawn_module, MessageType,
};
use anyhow::Context;

pub fn systems() -> SystemGroup {
    ambient_wasm::server::systems()
}

pub async fn initialize(
    world: &mut World,
    project_path: AbsAssetUrl,
    manifest: &ambient_project::Manifest,
    build_metadata: &ambient_build::Metadata,
) -> anyhow::Result<()> {
    let messenger = Arc::new(
        |world: &World, id: EntityId, type_: MessageType, message: &str| {
            let name = get_module_name(world, id);
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

    ambient_wasm::server::initialize(world, project_path.clone(), messenger)?;

    let build_dir = project_path.push("build").unwrap();

    let mut modules_to_entity_ids = HashMap::new();
    for target in ["client", "server"] {
        let wasm_component_paths: &[String] = build_metadata.component_paths(target);

        let is_sole_module = wasm_component_paths.len() == 1;
        for path in wasm_component_paths {
            let component_url = build_dir.push(path).unwrap();
            let name = Identifier::new(
                component_url
                    .file_stem()
                    .context("no file stem for {path:?}")?,
            )
            .map_err(anyhow::Error::msg)?;

            let description = manifest.ember.description.clone().unwrap_or_default();
            let description = if is_sole_module {
                description
            } else {
                format!("{description} ({name})")
            };

            let id = spawn_module(world, &name, description, true, target == "server");
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

            let bytecode_url = AbsAssetUrl::from_asset_key(path)?.to_string();
            world.add_component(id, bytecode_from_url(), bytecode_url)?;
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
