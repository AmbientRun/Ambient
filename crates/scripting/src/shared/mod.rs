pub(crate) mod implementation;
pub mod util;

pub(crate) mod bindings;
pub mod conversion;
pub mod guest_conversion;
pub mod host_guest_state;
pub mod interface;

mod script_module;
use std::sync::Arc;

use host_guest_state::GetBaseHostGuestState;
use itertools::Itertools;
use kiwi_ecs::{
    components, query, uid, uid_lookup, Component, EntityData, EntityId, Networked, Resource,
    Store, World,
};
use kiwi_project::Identifier;
use parking_lot::RwLock;
pub use script_module::*;
use util::get_module_name;
use wasi_common::WasiCtx;
use wasmtime::Linker;

components!("scripting::shared", {
    @[Networked, Store]
    script_module: (),
    @[Store]
    script_module_bytecode: ScriptModuleBytecode,
    @[Networked, Store]
    script_module_enabled: bool,
    @[Networked, Store]
    script_module_errors: ScriptModuleErrors,

    /// used to signal messages from the scripting host/runtime
    @[Resource]
    messenger: Arc<dyn Fn(&World, EntityId, MessageType, &str) + Send + Sync>,
});

pub const MAXIMUM_ERROR_COUNT: usize = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessageType {
    Info,
    Error,
    Stdout,
    Stderr,
}

#[derive(Debug, Clone)]
pub struct ScriptContext {
    pub event_name: String,
    pub event_data: EntityData,
    pub time: f32,
    pub frametime: f32,
}
impl ScriptContext {
    pub fn new(world: &World, event_name: &str, event_data: EntityData) -> Self {
        let time = kiwi_app::get_time_since_app_start(world).as_secs_f32();
        let frametime = *world.resource(kiwi_core::dtime());

        Self {
            event_name: event_name.to_string(),
            event_data,
            time,
            frametime,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn initialize(
    world: &mut World,
    messenger: Arc<dyn Fn(&World, EntityId, MessageType, &str) + Send + Sync>,
) -> anyhow::Result<()> {
    world.add_resource(self::messenger(), messenger);

    Ok(())
}

pub fn reload_all<
    Bindings: Send + Sync + 'static,
    Context: WasmContext<Bindings> + Send + Sync + 'static,
    HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
>(
    world: &mut World,
    state_component: Component<ScriptModuleState<Bindings, Context, HostGuestState>>,
    make_wasm_context: Arc<dyn Fn(WasiCtx, Arc<RwLock<HostGuestState>>) -> Context + Send + Sync>,
    add_to_linker: Arc<dyn Fn(&mut Linker<Context>) -> anyhow::Result<()> + Send + Sync>,
) {
    let scripts = query((
        script_module(),
        script_module_bytecode(),
        script_module_enabled(),
    ))
    .iter(world, None)
    .map(|(id, (_, bc, enabled))| (id, enabled.then(|| bc.clone())))
    .collect_vec();

    for (script_id, bytecode) in scripts {
        reload(
            world,
            state_component,
            make_wasm_context.clone(),
            add_to_linker.clone(),
            script_id,
            bytecode,
        );
    }
}

pub fn run_all<
    Bindings: Send + Sync + 'static,
    Context: WasmContext<Bindings> + Send + Sync + 'static,
    HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
>(
    world: &mut World,
    state_component: Component<ScriptModuleState<Bindings, Context, HostGuestState>>,
    context: &ScriptContext,
) {
    let errors: Vec<(EntityId, String)> = query(state_component)
        .collect_cloned(world, None)
        .into_iter()
        .flat_map(|(id, sms)| run(world, state_component, id, sms, context))
        .collect();
    update_errors(world, state_component, &errors, true);
}

pub fn reload<
    Bindings: Send + Sync + 'static,
    Context: WasmContext<Bindings> + Send + Sync + 'static,
    HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
>(
    world: &mut World,
    state_component: Component<ScriptModuleState<Bindings, Context, HostGuestState>>,
    make_wasm_context: Arc<dyn Fn(WasiCtx, Arc<RwLock<HostGuestState>>) -> Context + Send + Sync>,
    add_to_linker: Arc<dyn Fn(&mut Linker<Context>) -> anyhow::Result<()> + Send + Sync>,
    script_id: EntityId,
    bytecode: Option<ScriptModuleBytecode>,
) {
    let mut errors = unload(world, state_component, script_id, "reloading");

    if let Some(bytecode) = bytecode {
        if !bytecode.0.is_empty() {
            load(
                world,
                state_component,
                script_id,
                make_wasm_context.clone(),
                add_to_linker.clone(),
                &bytecode.0,
                &mut errors,
            );
        }
    }

    update_errors(world, state_component, &errors, true);
}

#[allow(clippy::too_many_arguments)]
pub fn load<
    Bindings: Send + Sync + 'static,
    Context: WasmContext<Bindings> + Send + Sync + 'static,
    HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
>(
    world: &mut World,
    state_component: Component<ScriptModuleState<Bindings, Context, HostGuestState>>,
    script_id: EntityId,
    make_wasm_context: Arc<dyn Fn(WasiCtx, Arc<RwLock<HostGuestState>>) -> Context + Send + Sync>,
    add_to_linker: Arc<dyn Fn(&mut Linker<Context>) -> anyhow::Result<()> + Send + Sync>,
    bytecode: &[u8],
    errors: &mut Vec<(EntityId, String)>,
) {
    let messenger = world.resource(messenger()).clone();
    let result = run_and_catch_panics(|| {
        ScriptModuleState::new(
            bytecode,
            Box::new({
                let messenger = messenger.clone();
                move |world, msg| {
                    messenger(world, script_id, MessageType::Stdout, msg);
                }
            }),
            Box::new(move |world, msg| {
                messenger(world, script_id, MessageType::Stderr, msg);
            }),
            move |ctx, state| make_wasm_context(ctx, state),
            move |linker| add_to_linker(linker),
            crate::shared::interface::shared::INTERFACE_VERSION,
        )
    });

    match result {
        Ok(sms) => {
            // Run the initial startup event.
            errors.extend(run(
                world,
                state_component,
                script_id,
                sms.clone(),
                &ScriptContext::new(world, "core/module_load", EntityData::new()),
            ));

            world
                .add_component(script_id, state_component, sms)
                .unwrap();
        }
        Err(err) => errors.push((script_id, err)),
    }
}

pub fn unload<
    Bindings: Send + Sync + 'static,
    Context: WasmContext<Bindings> + Send + Sync + 'static,
    HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
>(
    world: &mut World,
    state_component: Component<ScriptModuleState<Bindings, Context, HostGuestState>>,
    script_id: EntityId,
    reason: &str,
) -> Vec<(EntityId, String)> {
    let Ok(sms) = world.get_cloned(script_id, state_component) else { return vec![]; };

    let errors = run(
        world,
        state_component,
        script_id,
        sms,
        &ScriptContext::new(world, "core/module_unload", EntityData::new()),
    )
    .into_iter()
    .collect_vec();

    let spawned_entities = world
        .get_mut(script_id, state_component)
        .map(|sms| std::mem::take(&mut sms.shared_state().write().base_mut().spawned_entities))
        .unwrap_or_default();

    if let Ok(script_module_errors) = world.get_mut(script_id, script_module_errors()) {
        script_module_errors.runtime.clear();
    }

    world.remove_component(script_id, state_component).unwrap();

    for uid in spawned_entities {
        if let Ok(id) = world.resource(uid_lookup()).get(&uid) {
            world.despawn(id);
        }
    }

    let messenger = world.resource(messenger()).clone();
    messenger(
        world,
        script_id,
        MessageType::Info,
        &format!("Unloaded (reason: {reason})"),
    );

    errors
}

pub fn update_errors<
    Bindings: Send + Sync + 'static,
    Context: WasmContext<Bindings> + Send + Sync + 'static,
    HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
>(
    world: &mut World,
    state_component: Component<ScriptModuleState<Bindings, Context, HostGuestState>>,
    errors: &[(EntityId, String)],
    runtime: bool,
) {
    let messenger = world.resource(messenger()).clone();
    for (id, err) in errors {
        messenger(
            world,
            *id,
            MessageType::Error,
            &format!(
                "{} error: {}",
                match runtime {
                    true => "Run",
                    false => "Compile",
                },
                err
            ),
        );

        if let Ok(script_module_errors) = world.get_mut(*id, script_module_errors()) {
            let error_stream = match runtime {
                true => &mut script_module_errors.runtime,
                false => &mut script_module_errors.compiletime,
            };
            error_stream.push(err.clone());
            if error_stream.len() > MAXIMUM_ERROR_COUNT {
                unload(world, state_component, *id, "too many errors");
            }
        }
    }
}

pub fn run<
    Bindings: Send + Sync + 'static,
    Context: WasmContext<Bindings> + Send + Sync + 'static,
    HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
>(
    world: &mut World,
    state_component: Component<ScriptModuleState<Bindings, Context, HostGuestState>>,
    id: EntityId,
    mut state: ScriptModuleState<Bindings, Context, HostGuestState>,
    context: &ScriptContext,
) -> Option<(EntityId, String)> {
    profiling::scope!(
        "run_script",
        format!("{} - {}", get_module_name(world, id), context.event_name)
    );

    // If this is not a whitelisted event and it's not in the subscribed events,
    // skip over it
    if !["core/module_load", "core/frame"].contains(&context.event_name.as_str())
        && !state
            .shared_state
            .write()
            .base_mut()
            .event
            .subscribed_events
            .contains(&context.event_name)
    {
        return None;
    }

    let result = run_and_catch_panics(|| state.run(world, context));
    let events_to_run = std::mem::take(&mut state.shared_state.write().base_mut().event.events);
    world.set(id, state_component, state).ok();

    let err = result.err().map(|err| (id, err));
    // TODO(philpax): come up with a more intelligent dispatch scheme than this
    // This can very easily result in an infinite loop.
    // Things to fix include:
    // - don't let a script trigger an event on itself
    // - don't let two scripts chat with each other indefinitely (shunt them to the next tick)
    // - don't do the event dispatch in this function and instead do it *after* initial
    //   execution of all scripts
    for (event_name, event_data) in events_to_run {
        run_all(
            world,
            state_component,
            &ScriptContext::new(world, &event_name, event_data),
        );
    }

    err
}

pub fn spawn_script(
    world: &mut World,
    name: &Identifier,
    description: String,
    enabled: bool,
) -> anyhow::Result<EntityId> {
    if query(())
        .incl(script_module())
        .iter(world, None)
        .any(|(id, _)| &get_module_name(world, id) == name)
    {
        anyhow::bail!("a script module by the name {name} already exists");
    }

    let ed = EntityData::new()
        .set(kiwi_core::name(), name.to_string())
        .set(uid(), kiwi_ecs::EntityUid::create())
        .set_default(script_module())
        .set(script_module_enabled(), enabled)
        .set_default(script_module_errors())
        .set(kiwi_project::description(), description);

    Ok(ed.spawn(world))
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
