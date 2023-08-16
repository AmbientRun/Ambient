pub(crate) mod bindings;
pub(crate) mod engine;
pub(crate) mod implementation;

mod module;

pub mod build;
pub mod conversion;
pub mod host_guest_state;
pub mod message;
pub mod wit;

pub use ambient_ecs::generated::components::core::wasm::*;
pub use internal::{
    messenger, module_bytecode, module_errors, module_state, module_state_maker, remote_paired_id,
};
pub use module::*;

use std::{path::Path, str::FromStr, sync::Arc};

use ambient_core::{asset_cache, async_ecs::async_run, runtime};
use ambient_ecs::{
    dont_despawn_on_unload, generated::messages, query, world_events, Entity, EntityId, FnSystem,
    Message, SystemGroup, World, WorldEventReader,
};

use ambient_native_std::{
    asset_cache::AssetCache, asset_url::AbsAssetUrl, download_asset::download_uncached_bytes,
};
use itertools::Itertools;
#[cfg(not(target_os = "unknown"))]
use wasi_cap_std_sync::Dir;

mod internal {
    use std::sync::Arc;

    use ambient_ecs::{
        components, Debuggable, Description, EntityId, Networked, Resource, Store, World,
    };

    use super::{MessageType, ModuleBytecode, ModuleErrors, ModuleState, ModuleStateArgs};

    components!("wasm::shared", {
        module_state: ModuleState,
        @[Store, Description["Bytecode of a WASM component; if attached, will be run."]]
        module_bytecode: ModuleBytecode,
        @[Networked, Store, Debuggable]
        module_errors: ModuleErrors,
        @[Networked, Debuggable, Description["The ID of the module on the \"other side\" of this module, if available. (e.g. serverside module to clientside module)."]]
        remote_paired_id: EntityId,

        @[Resource, Description["Used to signal messages from the WASM host/runtime."]]
        messenger: Arc<dyn Fn(&World, EntityId, MessageType, &str) + Send + Sync>,
        @[Resource]
        module_state_maker: Arc<dyn Fn(ModuleStateArgs<'_>) -> anyhow::Result<ModuleState> + Sync + Send>,
    });
}

#[cfg(not(target_os = "unknown"))]
mod native_bindings {
    use std::sync::Arc;

    use ambient_ecs::{components, Resource};
    use wasi_cap_std_sync::Dir;

    components!("wasm::shared", {
        @[Resource]
        preopened_dir: Arc<Dir>,
    });
}

use self::message::Source;
#[cfg(not(target_os = "unknown"))]
use self::native_bindings::preopened_dir;
use crate::shared::message::{RuntimeMessageExt, Target};

pub fn init_all_components() {
    internal::init_components();
    #[cfg(not(target_os = "unknown"))]
    native_bindings::init_components();
    message::init_components();
}

pub const MAXIMUM_ERROR_COUNT: usize = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessageType {
    Info,
    Warn,
    Error,
    Stdout,
    Stderr,
}

pub fn systems() -> SystemGroup {
    let mut app_events_reader = WorldEventReader::new();

    SystemGroup::new(
        "core/wasm",
        vec![
            query(bytecode_from_url())
                .incl(module())
                .excl(module_name())
                .spawned()
                .to_system(move |q, world, qs, _| {
                    for (id, url) in q.collect_cloned(world, qs) {
                        let url = AbsAssetUrl::from_str(&url).unwrap();
                        let decoded_path = url.decoded_path();
                        let name = decoded_path
                            .file_stem()
                            .unwrap_or_else(|| decoded_path.as_str());

                        world
                            .add_component(id, module_name(), name.to_string())
                            .ok();
                    }
                }),
            query(module_name())
                .incl(module())
                .excl(ambient_core::name())
                .spawned()
                .to_system(move |q, world, qs, _| {
                    for (id, name) in q.collect_cloned(world, qs) {
                        world
                            .add_component(
                                id,
                                ambient_core::name(),
                                format!("Wasm module: {}", name),
                            )
                            .ok();
                    }
                }),
            query(module())
                .excl(module_errors())
                .spawned()
                .to_system(move |q, world, qs, _| {
                    for (id, _) in q.collect_cloned(world, qs) {
                        world
                            .add_component(id, module_errors(), Default::default())
                            .ok();
                    }
                }),
            query(bytecode_from_url().changed()).to_system(move |q, world, qs, _| {
                // TODO: there has got to be a better way to do this
                let is_server = world.name() == "server";

                for (id, url) in q.collect_cloned(world, qs) {
                    let on_server = world.has_component(id, module_on_server());
                    if is_server != on_server {
                        continue;
                    }

                    let url = match AbsAssetUrl::from_str(&url) {
                        Ok(value) => value,
                        Err(err) => {
                            log::warn!("Failed to parse bytecode_from_url URL: {:?}", err);
                            continue;
                        }
                    };
                    let assets = world.resource(asset_cache()).clone();
                    let async_run = world.resource(async_run()).clone();
                    world.resource(runtime()).spawn(async move {
                        // TODO: We use an uncached download here to ensure that we can
                        // reload modules on the fly. Revisit once we have some form of
                        // hot-reloading working.
                        match download_uncached_bytes(&assets, url.clone()).await {
                            Err(err) => {
                                log::warn!("Failed to load bytecode from URL: {:?}", err);
                            }
                            Ok(bytecode) => {
                                async_run.run(move |world| {
                                    world
                                        .add_component(
                                            id,
                                            module_bytecode(),
                                            ModuleBytecode(bytecode.to_vec()),
                                        )
                                        .ok();
                                });
                            }
                        }
                    });
                }
            }),
            query((module_bytecode().changed(), module_enabled().changed())).to_system(
                move |q, world, qs, _| {
                    profiling::scope!("WASM module reloads");
                    let modules = q
                        .iter(world, qs)
                        .map(|(id, (bytecode, enabled))| (id, enabled.then(|| bytecode.clone())))
                        .collect_vec();

                    for (id, bytecode) in modules {
                        reload(world, id, bytecode);
                    }
                },
            ),
            Box::new(FnSystem::new(move |world, _| {
                profiling::scope!("WASM module app events");
                let events = app_events_reader
                    .iter(world.resource(world_events()))
                    .map(|(_, event)| event.clone())
                    .collect_vec();

                for (name, data) in events {
                    message::run(
                        world,
                        message::SerializedMessage {
                            target: Target::All { include_self: true },
                            source: message::Source::Runtime,
                            name,
                            data,
                        },
                    );
                }
            })),
            Box::new(FnSystem::new(move |world, _| {
                profiling::scope!("WASM module frame event");
                // trigger frame event
                ambient_ecs::generated::messages::Frame::new()
                    .run(world, None)
                    .unwrap();
            })),
            Box::new(FnSystem::new(move |world, _| {
                profiling::scope!("WASM module pending messages");

                let pending_messages =
                    std::mem::take(world.resource_mut(message::pending_messages()));

                for message in pending_messages {
                    message::run(world, message);
                }
            })),
        ],
    )
}

/// Initialize the core of the WASM runtime
pub fn initialize<'a, Bindings: bindings::BindingsBound + 'static>(
    world: &mut World,
    assets: &AssetCache,
    messenger: Arc<dyn Fn(&World, EntityId, MessageType, &str) + Send + Sync>,
    bindings: fn(EntityId) -> Bindings,
    preopened_dir_path: Option<&'a Path>,
) -> anyhow::Result<()> {
    world.add_resource(self::messenger(), messenger);
    world.add_resource(
        self::module_state_maker(),
        ModuleState::create_state_maker(assets, bindings),
    );

    world.add_resource(message::pending_messages(), vec![]);

    #[cfg(not(target_os = "unknown"))]
    if let Some(preopened_dir_path) = preopened_dir_path {
        std::fs::create_dir_all(preopened_dir_path)?;
        world.add_resource(
            preopened_dir(),
            Arc::new(Dir::open_ambient_dir(
                preopened_dir_path,
                wasi_cap_std_sync::ambient_authority(),
            )?),
        );
    }

    Ok(())
}

pub(crate) fn reload_all(world: &mut World) {
    let modules = query((module(), module_bytecode(), module_enabled()))
        .iter(world, None)
        .map(|(id, (_, bc, enabled))| (id, enabled.then(|| bc.clone())))
        .collect_vec();

    for (module_id, bytecode) in modules {
        reload(world, module_id, bytecode);
    }
}

fn reload(world: &mut World, module_id: EntityId, new_bytecode: Option<ModuleBytecode>) {
    unload(world, module_id, "reloading");

    if let Some(new_bytecode) = new_bytecode {
        if !new_bytecode.0.is_empty() {
            load(world, module_id, &new_bytecode.0);
        }
    }
}

/// Loads a wasm module from the given bytecode and attaches it to the given entity.
fn load(world: &mut World, id: EntityId, component_bytecode: &[u8]) {
    let messenger = world.resource(messenger()).clone();
    let module_state_maker = world.resource(module_state_maker()).clone();

    let async_run = world.resource(async_run()).clone();
    let component_bytecode = component_bytecode.to_vec();
    let name = world
        .get_ref(id, module_name())
        .map(|x| x.clone())
        .unwrap_or_else(|_| "Unknown".to_string());

    #[cfg(not(target_os = "unknown"))]
    let preopened_dir = world
        .resource_opt(preopened_dir())
        .map(|d| d.try_clone().unwrap());

    // Spawn the module on another thread to ensure that it does not block the main thread during compilation.
    std::thread::spawn(move || {
        let result = run_and_catch_panics(|| {
            log::info!("Loading module: {}", name);
            let res = module_state_maker(module::ModuleStateArgs {
                component_bytecode: &component_bytecode,
                stdout_output: Box::new({
                    let messenger = messenger.clone();
                    move |world, msg| {
                        messenger(world, id, MessageType::Stdout, msg);
                    }
                }),
                stderr_output: Box::new(move |world, msg| {
                    messenger(world, id, MessageType::Stderr, msg);
                }),
                id,
                #[cfg(not(target_os = "unknown"))]
                preopened_dir,
            });
            log::info!("Done loading module: {}", name);
            res
        });

        async_run.run(move |world| {
            match result {
                Ok(mut sms) => {
                    // Subscribe the module to messages that it should be aware of.
                    let autosubscribe_messages =
                        [messages::Frame::id(), messages::ModuleLoad::id()];
                    for id in autosubscribe_messages {
                        sms.listen_to_message(id.to_string());
                    }

                    world.add_component(id, module_state(), sms).unwrap();

                    log::info!("Running startup event for module: {}", name);
                    messages::ModuleLoad::new().run(world, Some(id)).unwrap();
                }
                Err(err) => update_errors(world, &[(id, err)]),
            }
        });
    });
}

fn update_errors(world: &mut World, errors: &[(EntityId, String)]) {
    let messenger = world.resource(messenger()).clone();
    for (id, err) in errors {
        messenger(
            world,
            *id,
            MessageType::Error,
            &format!("Runtime error: {}", err),
        );

        if let Ok(module_errors) = world.get_mut(*id, module_errors()) {
            let error_stream = &mut module_errors.0;

            error_stream.push(err.clone());
            if error_stream.len() > MAXIMUM_ERROR_COUNT {
                unload(world, *id, "too many errors");
            }
        }
    }
}

fn run(
    world: &mut World,
    id: EntityId,
    mut state: ModuleState,
    message_source: &Source,
    message_name: &str,
    message_data: &[u8],
) {
    profiling::scope!(
        "run",
        format!(
            "{} - {}",
            world.get_cloned(id, module_name()).unwrap_or_default(),
            message_name
        )
        .as_str()
    );

    // If it's not in the subscribed events, skip over it
    if !state.supports_message(message_name) {
        return;
    }

    let result =
        run_and_catch_panics(|| state.run(world, message_source, message_name, message_data));

    if let Err(message) = result {
        update_errors(world, &[(id, message)]);
    }
}

pub(crate) fn unload(world: &mut World, module_id: EntityId, reason: &str) {
    if !world.has_component(module_id, module_state()) {
        return;
    }

    messages::ModuleUnload::new()
        .run(world, Some(module_id))
        .unwrap();

    let spawned_entities = world
        .get_mut(module_id, module_state())
        .map(|sms| sms.drain_spawned_entities())
        .unwrap_or_default();

    if let Ok(module_errors) = world.get_mut(module_id, module_errors()) {
        module_errors.0.clear();
    }

    world.remove_component(module_id, module_state()).unwrap();

    for id in spawned_entities {
        if !world.has_component(id, dont_despawn_on_unload()) {
            world.despawn(id);
        }
    }

    let messenger = world.resource(messenger()).clone();
    messenger(
        world,
        module_id,
        MessageType::Info,
        &format!("Unloaded (reason: {reason})"),
    );
}

pub fn spawn_module(
    world: &mut World,
    bytecode_from_url: AbsAssetUrl,
    enabled: bool,
    on_server: bool,
) -> EntityId {
    let entity = Entity::new()
        .with_default(module())
        .with(self::bytecode_from_url(), bytecode_from_url.to_string())
        .with(module_enabled(), enabled);

    let entity = if on_server {
        entity.with_default(module_on_server())
    } else {
        entity
    };

    entity.spawn(world)
}

fn run_and_catch_panics<R>(f: impl FnOnce() -> anyhow::Result<R>) -> Result<R, String> {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    match result {
        Ok(Ok(r)) => Ok(r),
        Ok(Err(e)) => {
            let mut output = e.to_string();
            let root_cause = e.root_cause().to_string();
            if root_cause != output {
                output = format!("{output}\nRoot cause: {root_cause}");
            }
            Err(output)
        }
        Err(e) => Err(match e.downcast::<String>() {
            Ok(e) => format!("{e}"),
            Err(e) => match e.downcast::<&str>() {
                Ok(e) => format!("{e}"),
                _ => "unknown error".to_string(),
            },
        }),
    }
}
