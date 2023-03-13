pub(crate) mod bindings;
mod borrowed_types;
pub mod build;
pub mod conversion;
pub mod host_guest_state;
pub(crate) mod implementation;
mod module;
pub mod wit;

use std::sync::Arc;

use ambient_ecs::{
    components, dont_despawn_on_unload, query, world_events, ComponentEntry, Debuggable,
    Description, Entity, EntityId, FnSystem, Networked, Resource, Store, SystemGroup, World,
    WorldEventReader,
};
use ambient_physics::{collider_loads, collisions, PxShapeUserData};
use ambient_project::Identifier;
use itertools::Itertools;
pub use module::*;
use physxx::{PxRigidActor, PxRigidActorRef, PxUserData};

components!("wasm::shared", {
    @[Networked, Store, Debuggable]
    module: (),
    module_state: ModuleState,
    @[Store, Description["Bytecode of a WASM component; if attached, will be run."]]
    module_bytecode: ModuleBytecode,
    @[Store, Networked, Description["Bytecode of a clientside WASM component. On the client, this will be copied over to `module_bytecode` automatically."]]
    client_module_bytecode: ModuleBytecode,
    @[Networked, Store, Debuggable]
    module_enabled: bool,
    @[Networked, Store, Debuggable]
    module_errors: ModuleErrors,

    @[Resource, Description["Used to signal messages from the WASM host/runtime."]]
    messenger: Arc<dyn Fn(&World, EntityId, MessageType, &str) + Send + Sync>,
    @[Resource]
    module_state_maker: Arc<dyn Fn(ModuleStateArgs<'_>) -> anyhow::Result<ModuleState> + Sync + Send>,
});

pub const MAXIMUM_ERROR_COUNT: usize = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessageType {
    Info,
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
    pub fn new(world: &World, event_name: &str, event_data: Entity) -> Self {
        let time = ambient_app::get_time_since_app_start(world).as_secs_f32();

        Self {
            event_name: event_name.to_string(),
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

                for event in events {
                    run_all(world, &RunContext::new(world, "core/world_event", event));
                }
            })),
            Box::new(FnSystem::new(move |world, _| {
                profiling::scope!("WASM module frame event");
                // trigger frame event
                run_all(world, &RunContext::new(world, "core/frame", Entity::new()));
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
                            "core/collision",
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
                            "core/collider_load",
                            vec![ComponentEntry::new(ambient_ecs::id(), id)].into(),
                        ),
                    );
                }
            })),
        ],
    )
}

pub fn initialize<Bindings: bindings::BindingsBound + 'static>(
    world: &mut World,
    messenger: Arc<dyn Fn(&World, EntityId, MessageType, &str) + Send + Sync>,
    bindings: Bindings,
) -> anyhow::Result<()> {
    world.add_resource(self::messenger(), messenger);
    world.add_resource(
        self::module_state_maker(),
        ModuleState::create_state_maker(bindings),
    );

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

pub fn run_all(world: &mut World, context: &RunContext) {
    let errors: Vec<(EntityId, String)> = query(module_state())
        .collect_cloned(world, None)
        .into_iter()
        .flat_map(|(id, sms)| run(world, id, sms, context))
        .collect();

    update_errors(world, &errors);
}

fn reload(world: &mut World, module_id: EntityId, bytecode: Option<ModuleBytecode>) {
    let mut errors = unload(world, module_id, "reloading");

    if let Some(bytecode) = bytecode {
        if !bytecode.0.is_empty() {
            load(world, module_id, &bytecode.0, &mut errors);
        }
    }

    update_errors(world, &errors);
}

#[allow(clippy::too_many_arguments)]
fn load(
    world: &mut World,
    module_id: EntityId,
    component_bytecode: &[u8],
    errors: &mut Vec<(EntityId, String)>,
) {
    let messenger = world.resource(messenger()).clone();
    let module_state_maker = world.resource(module_state_maker()).clone();
    let result = run_and_catch_panics(|| {
        module_state_maker(module::ModuleStateArgs {
            component_bytecode,
            stdout_output: Box::new({
                let messenger = messenger.clone();
                move |world, msg| {
                    messenger(world, module_id, MessageType::Stdout, msg);
                }
            }),
            stderr_output: Box::new(move |world, msg| {
                messenger(world, module_id, MessageType::Stderr, msg);
            }),
        })
    });

    match result {
        Ok(sms) => {
            // Run the initial startup event.
            errors.extend(run(
                world,
                module_id,
                sms.clone(),
                &RunContext::new(world, "core/module_load", Entity::new()),
            ));

            world.add_component(module_id, module_state(), sms).unwrap();
        }
        Err(err) => errors.push((module_id, err)),
    }
}

pub(crate) fn unload(
    world: &mut World,
    module_id: EntityId,
    reason: &str,
) -> Vec<(EntityId, String)> {
    let Ok(sms) = world.get_cloned(module_id, module_state()) else { return vec![]; };

    let errors = run(
        world,
        module_id,
        sms,
        &RunContext::new(world, "core/module_unload", Entity::new()),
    )
    .into_iter()
    .collect_vec();

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

    errors
}

pub(crate) fn update_errors(world: &mut World, errors: &[(EntityId, String)]) {
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
    context: &RunContext,
) -> Option<(EntityId, String)> {
    profiling::scope!(
        "run",
        format!("{} - {}", get_module_name(world, id), context.event_name)
    );

    // If this is not a whitelisted event and it's not in the subscribed events,
    // skip over it
    if !["core/module_load", "core/frame"].contains(&context.event_name.as_str())
        && !state.supports_event(&context.event_name)
    {
        return None;
    }

    let result = run_and_catch_panics(|| state.run(world, context));
    world.set(id, module_state(), state).ok();

    result.err().map(|err| (id, err))
}

pub fn spawn_module(
    world: &mut World,
    name: &Identifier,
    description: String,
    enabled: bool,
) -> anyhow::Result<EntityId> {
    let ed = Entity::new()
        .with(ambient_core::name(), name.to_string())
        .with_default(module())
        .with(module_enabled(), enabled)
        .with_default(module_errors())
        .with(ambient_project::description(), description);

    Ok(ed.spawn(world))
}

pub fn get_module_name(world: &World, id: EntityId) -> Identifier {
    Identifier::new(world.get_cloned(id, ambient_core::name()).unwrap()).unwrap()
}

fn run_and_catch_panics<R>(f: impl FnOnce() -> anyhow::Result<R>) -> Result<R, String> {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    match result {
        Ok(Ok(r)) => Ok(r),
        Ok(Err(e)) => Err(e.to_string()),
        Err(e) => Err(match e.downcast::<String>() {
            Ok(e) => e.to_string(),
            Err(e) => match e.downcast::<&str>() {
                Ok(e) => e.to_string(),
                _ => "unknown error".to_string(),
            },
        }),
    }
}
