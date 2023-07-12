pub(crate) mod bindings;
pub(crate) mod implementation;

mod module;

pub mod build;
#[cfg(feature = "wit")]
pub mod conversion;
pub mod host_guest_state;
pub mod message;

#[cfg(feature = "wit")]
pub mod wit;

use std::sync::Arc;

use ambient_core::async_ecs::async_run;
use ambient_ecs::{
    dont_despawn_on_unload, generated::messages::core as messages, query, world_events, Entity,
    EntityId, FnSystem, Message, SystemGroup, World, WorldEventReader,
};

use ambient_project::Identifier;
use itertools::Itertools;
pub use module::*;

mod internal {
    use std::sync::Arc;

    use ambient_ecs::{
        components, Debuggable, Description, EntityId, Networked, Resource, Store, World,
    };

    use super::{MessageType, ModuleBytecode, ModuleErrors, ModuleState, ModuleStateArgs};

    components!("wasm::shared", {
        @[Networked, Store, Debuggable]
        module: (),
        module_state: ModuleState,
        @[Store, Description["Bytecode of a WASM component; if attached, will be run."]]
        module_bytecode: ModuleBytecode,
        @[Networked, Store, Debuggable, Description["Asset URL for the bytecode of a clientside WASM component."]]
        client_bytecode_from_url: String,
        @[Networked, Store, Debuggable]
        module_enabled: bool,
        @[Networked, Store, Debuggable]
        module_name: String,
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

pub use internal::{
    client_bytecode_from_url, messenger, module, module_bytecode, module_enabled, module_errors,
    module_state, module_state_maker, remote_paired_id,
};

use self::{internal::module_name, message::Source};
use crate::shared::message::{RuntimeMessageExt, Target};

pub fn init_all_components() {
    internal::init_components();
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
            query((module_bytecode(), module_enabled().changed())).to_system(
                move |q, world, qs, _| {
                    ambient_profiling::scope!("WASM module reloads");
                    let modules = q
                        .iter(world, qs)
                        .filter(|(id, (_, enabled))| {
                            let has_state = world.has_component(*id, module_state());
                            **enabled != has_state
                        })
                        .map(|(id, (bytecode, enabled))| (id, enabled.then(|| bytecode.clone())))
                        .collect_vec();

                    for (id, bytecode) in modules {
                        reload(world, id, bytecode);
                    }
                },
            ),
            Box::new(FnSystem::new(move |world, _| {
                ambient_profiling::scope!("WASM module app events");
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
                ambient_profiling::scope!("WASM module frame event");
                // trigger frame event
                messages::Frame::new().run(world, None).unwrap();
            })),
            Box::new(FnSystem::new(move |world, _| {
                ambient_profiling::scope!("WASM module pending messages");

                let pending_messages =
                    std::mem::take(world.resource_mut(message::pending_messages()));

                for message in pending_messages {
                    message::run(world, message);
                }
            })),
        ],
    )
}

#[cfg(feature = "wit")]
pub fn initialize<Bindings: bindings::BindingsBound + 'static>(
    world: &mut World,
    messenger: Arc<dyn Fn(&World, EntityId, MessageType, &str) + Send + Sync>,
    bindings: fn(EntityId) -> Bindings,
) -> anyhow::Result<()> {
    world.add_resource(self::messenger(), messenger);
    world.add_resource(
        self::module_state_maker(),
        ModuleState::create_state_maker(bindings),
    );
    world.add_resource(message::pending_messages(), vec![]);

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

fn reload(world: &mut World, module_id: EntityId, bytecode: Option<ModuleBytecode>) {
    unload(world, module_id, "reloading");

    if let Some(bytecode) = bytecode {
        if !bytecode.0.is_empty() {
            load(world, module_id, &bytecode.0);
        }
    }
}

fn load(world: &mut World, module_id: EntityId, component_bytecode: &[u8]) {
    let messenger = world.resource(messenger()).clone();
    let module_state_maker = world.resource(module_state_maker()).clone();

    let async_run = world.resource(async_run()).clone();
    let component_bytecode = component_bytecode.to_vec();
    let name = world
        .get_ref(module_id, module_name())
        .map(|x| x.clone())
        .unwrap_or_else(|_| "Unknown".to_string());

    // Spawn the module on another thread to ensure that it does not block the main thread during compilation.
    std::thread::spawn(move || {
        let result = run_and_catch_panics(|| {
            log::info!("Loading module: {}", name);
            let res = module_state_maker(module::ModuleStateArgs {
                component_bytecode: &component_bytecode,
                stdout_output: Box::new({
                    let messenger = messenger.clone();
                    move |world, msg| {
                        messenger(world, module_id, MessageType::Stdout, msg);
                    }
                }),
                stderr_output: Box::new(move |world, msg| {
                    messenger(world, module_id, MessageType::Stderr, msg);
                }),
                id: module_id,
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

                    world.add_component(module_id, module_state(), sms).unwrap();

                    log::info!("Running startup event for module: {}", name);
                    messages::ModuleLoad::new()
                        .run(world, Some(module_id))
                        .unwrap();
                }
                Err(err) => update_errors(world, &[(module_id, err)]),
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
    ambient_profiling::scope!(
        "run",
        format!("{} - {}", get_module_name(world, id), message_name)
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
    name: &Identifier,
    description: String,
    enabled: bool,
) -> EntityId {
    Entity::new()
        .with(ambient_core::name(), format!("Wasm module: {}", name))
        .with(module_name(), name.to_string())
        .with(ambient_core::description(), description)
        .with_default(module())
        .with(module_enabled(), enabled)
        .with_default(module_errors())
        .spawn(world)
}

pub fn get_module_name(world: &World, id: EntityId) -> Identifier {
    Identifier::new(world.get_cloned(id, module_name()).unwrap()).unwrap()
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
