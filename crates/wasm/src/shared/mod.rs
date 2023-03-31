pub(crate) mod bindings;
pub(crate) mod implementation;

mod borrowed_types;
mod module;

pub mod build;
pub mod conversion;
pub mod host_guest_state;
pub mod wit;

use std::sync::Arc;

use ambient_core::async_ecs::async_run;
use ambient_ecs::{
    dont_despawn_on_unload, query, world_events, ComponentEntry, Entity, EntityId, FnSystem,
    SystemGroup, World, WorldEventReader,
};
use ambient_physics::{collider_loads, collisions, PxShapeUserData};
use ambient_project::Identifier;
use ambient_shared_types::events;
use itertools::Itertools;
use physxx::{PxRigidActor, PxRigidActorRef, PxUserData};

pub use module::*;

mod internal {
    use ambient_ecs::{
        components, Debuggable, Description, EntityId, Networked, Resource, Store, World,
    };
    use std::sync::Arc;

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

pub mod message {
    use ambient_ecs::{components, Debuggable, Description, EntityId, Name, Resource, World};

    #[derive(Clone, PartialEq, Debug)]
    pub enum Source {
        Network,
        NetworkUserId(String),
        Module(EntityId),
    }

    #[derive(Clone, PartialEq, Debug)]
    pub struct PendingMessage {
        /// If unspecified, this will broadcast to all modules
        pub(super) module_id: Option<EntityId>,
        pub(super) source: Source,
        pub(super) name: String,
        pub(super) data: Vec<u8>,
    }

    pub fn send(
        world: &mut World,
        module_id: Option<EntityId>,
        source: Source,
        name: String,
        data: Vec<u8>,
    ) {
        world.resource_mut(pending_messages()).push(PendingMessage {
            module_id,
            source,
            name,
            data,
        });
    }

    components!("wasm::message", {
        @[Debuggable, Name["Source: Remote"], Description["This message came from the network with no specific source (likely the server)."]]
        source_remote: (),

        @[Debuggable, Name["Source: Remote (User ID)"], Description["This message came from this user."]]
        source_remote_user_id: String,

        @[Debuggable, Name["Source: Local"], Description["This message came from the specified module on this side."]]
        source_local: EntityId,

        @[Debuggable, Name["Data"], Description["The data payload of a message."]]
        data: Vec<u8>,

        @[Debuggable, Resource]
        pending_messages: Vec<PendingMessage>,
    });
}

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

#[derive(Debug, Clone)]
pub struct RunContext {
    pub event_name: String,
    pub event_data: Entity,
    pub time: f32,
}
impl RunContext {
    pub fn new(world: &World, event_name: impl Into<String>, event_data: Entity) -> Self {
        let time = ambient_app::get_time_since_app_start(world).as_secs_f32();

        Self {
            event_name: event_name.into(),
            event_data,
            time,
        }
    }
}

pub fn systems() -> SystemGroup {
    let mut app_events_reader = WorldEventReader::new();

    SystemGroup::new(
        "core/wasm",
        vec![
            query((module_bytecode(), module_enabled().changed())).to_system(
                move |q, world, qs, _| {
                    profiling::scope!("WASM module reloads");
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
                profiling::scope!("WASM module app events");
                let events = app_events_reader
                    .iter(world.resource(world_events()))
                    .map(|(_, event)| event.clone())
                    .collect_vec();

                for (name, data) in events {
                    run_all(world, &RunContext::new(world, &name, data));
                }
            })),
            Box::new(FnSystem::new(move |world, _| {
                profiling::scope!("WASM module frame event");
                // trigger frame event
                run_all(world, &RunContext::new(world, events::FRAME, Entity::new()));
            })),
            Box::new(FnSystem::new(move |world, _| {
                profiling::scope!("WASM module collision event");
                // trigger collision event
                let collisions = match world.resource_opt(collisions()) {
                    Some(collisions) => collisions.lock().clone(),
                    None => return,
                };
                for (a, b) in collisions.into_iter() {
                    let select_entity = |px: PxRigidActorRef| {
                        px.get_shapes()
                            .into_iter()
                            .next()
                            .and_then(|shape| shape.get_user_data::<PxShapeUserData>())
                            .map(|ud| ud.entity)
                    };

                    let ids = [select_entity(a), select_entity(b)]
                        .into_iter()
                        .flatten()
                        .collect_vec();

                    run_all(
                        world,
                        &RunContext::new(
                            world,
                            events::COLLISION,
                            vec![ComponentEntry::new(ambient_ecs::ids(), ids)].into(),
                        ),
                    );
                }
            })),
            Box::new(FnSystem::new(move |world, _| {
                profiling::scope!("WASM module collider loads");
                // trigger collider loads
                let collider_loads = match world.resource_opt(collider_loads()) {
                    Some(collider_loads) => collider_loads.clone(),
                    None => return,
                };
                for id in collider_loads {
                    run_all(
                        world,
                        &RunContext::new(
                            world,
                            events::COLLIDER_LOAD,
                            vec![ComponentEntry::new(ambient_ecs::id(), id)].into(),
                        ),
                    );
                }
            })),
            Box::new(FnSystem::new(move |world, _| {
                use message::{PendingMessage, Source};

                profiling::scope!("WASM module pending messages");

                let pending_messages =
                    std::mem::take(world.resource_mut(message::pending_messages()));

                for PendingMessage {
                    module_id,
                    source,
                    name,
                    data,
                } in pending_messages
                {
                    let mut entity = Entity::new().with(message::data(), data.to_vec());

                    let mut source_id = None;
                    match &source {
                        Source::Network => entity.set(message::source_remote(), ()),
                        Source::NetworkUserId(user_id) => {
                            entity.set(message::source_remote_user_id(), user_id.to_owned())
                        }
                        Source::Module(id) => {
                            source_id = Some(*id);
                            entity.set(message::source_local(), *id);
                        }
                    };

                    let run_context = RunContext::new(
                        world,
                        format!("{}/{}", events::MODULE_MESSAGE, name),
                        entity,
                    );

                    if let Some(module_id) = module_id {
                        match world.get_cloned(module_id, module_state()) {
                            Ok(state) => run(world, module_id, state, &run_context),
                            Err(_) => {
                                let module_name = world
                                    .get_cloned(module_id, ambient_core::name())
                                    .unwrap_or_default();

                                world.resource(messenger()).as_ref()(
                                    world, module_id, MessageType::Warn,
                                    &format!("Received message for unloaded module {module_id} ({module_name}); message {name:?} from {source:?}")
                                );
                            }
                        }
                    } else {
                        for (id, sms) in query(module_state()).collect_cloned(world, None) {
                            if Some(id) == source_id {
                                continue;
                            }

                            run(world, id, sms, &run_context)
                        }
                    }
                }
            })),
        ],
    )
}

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

fn run_all(world: &mut World, context: &RunContext) {
    for (id, sms) in query(module_state()).collect_cloned(world, None) {
        run(world, id, sms, context)
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

    // Spawn the module on another thread to ensure that it does not block the main thread during compilation.
    std::thread::spawn(move || {
        let result = run_and_catch_panics(|| {
            module_state_maker(module::ModuleStateArgs {
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
            })
        });

        async_run.run(move |world| {
            match result {
                Ok(sms) => {
                    // Run the initial startup event.
                    run(
                        world,
                        module_id,
                        sms.clone(),
                        &RunContext::new(world, events::MODULE_LOAD, Entity::new()),
                    );
                    world.add_component(module_id, module_state(), sms).unwrap();
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

fn run(world: &mut World, id: EntityId, mut state: ModuleState, context: &RunContext) {
    profiling::scope!(
        "run",
        format!("{} - {}", get_module_name(world, id), context.event_name)
    );

    // If this is not a whitelisted event and it's not in the subscribed events,
    // skip over it
    if !["core/module_load", "core/frame"].contains(&context.event_name.as_str())
        && !state.supports_event(&context.event_name)
    {
        return;
    }

    let result = run_and_catch_panics(|| state.run(world, context));

    if let Err(message) = result {
        update_errors(world, &[(id, message)]);
    }
}

pub(crate) fn unload(world: &mut World, module_id: EntityId, reason: &str) {
    let Ok(sms) = world.get_cloned(module_id, module_state()) else { return; };

    run(
        world,
        module_id,
        sms,
        &RunContext::new(world, events::MODULE_UNLOAD, Entity::new()),
    );

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
        .with(ambient_core::name(), name.to_string())
        .with_default(module())
        .with(module_enabled(), enabled)
        .with_default(module_errors())
        .with(ambient_project::description(), description)
        .spawn(world)
}

pub fn get_module_name(world: &World, id: EntityId) -> Identifier {
    Identifier::new(world.get_cloned(id, ambient_core::name()).unwrap()).unwrap()
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
