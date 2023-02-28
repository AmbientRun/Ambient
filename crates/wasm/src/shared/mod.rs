pub(crate) mod bindings;
pub mod conversion;
pub mod guest_conversion;
pub mod host_guest_state;
pub(crate) mod implementation;
pub mod interface;

mod module;
use std::sync::Arc;

use ambient_ecs::{
    components, dont_despawn_on_unload, query, Component, Entity, EntityId, Networked, Resource,
    Store, World,
};
use ambient_project::Identifier;
use host_guest_state::GetBaseHostGuestState;
use itertools::Itertools;
pub use module::*;
use parking_lot::RwLock;
use wasi_common::WasiCtx;
use wasmtime::Linker;

components!("wasm::shared", {
    @[Networked, Store]
    module: (),
    @[Store]
    module_bytecode: ModuleBytecode,
    @[Networked, Store]
    module_enabled: bool,
    @[Networked, Store]
    module_errors: ModuleErrors,

    /// used to signal messages from the WASM host/runtime
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
    state_component: Component<ModuleState<Bindings, Context, HostGuestState>>,
    make_wasm_context: Arc<dyn Fn(WasiCtx, Arc<RwLock<HostGuestState>>) -> Context + Send + Sync>,
    add_to_linker: Arc<dyn Fn(&mut Linker<Context>) -> anyhow::Result<()> + Send + Sync>,
) {
    let modules = query((module(), module_bytecode(), module_enabled()))
        .iter(world, None)
        .map(|(id, (_, bc, enabled))| (id, enabled.then(|| bc.clone())))
        .collect_vec();

    for (module_id, bytecode) in modules {
        reload(
            world,
            state_component,
            make_wasm_context.clone(),
            add_to_linker.clone(),
            module_id,
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
    state_component: Component<ModuleState<Bindings, Context, HostGuestState>>,
    context: &RunContext,
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
    state_component: Component<ModuleState<Bindings, Context, HostGuestState>>,
    make_wasm_context: Arc<dyn Fn(WasiCtx, Arc<RwLock<HostGuestState>>) -> Context + Send + Sync>,
    add_to_linker: Arc<dyn Fn(&mut Linker<Context>) -> anyhow::Result<()> + Send + Sync>,
    module_id: EntityId,
    bytecode: Option<ModuleBytecode>,
) {
    let mut errors = unload(world, state_component, module_id, "reloading");

    if let Some(bytecode) = bytecode {
        if !bytecode.0.is_empty() {
            load(
                world,
                state_component,
                module_id,
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
    state_component: Component<ModuleState<Bindings, Context, HostGuestState>>,
    module_id: EntityId,
    make_wasm_context: Arc<dyn Fn(WasiCtx, Arc<RwLock<HostGuestState>>) -> Context + Send + Sync>,
    add_to_linker: Arc<dyn Fn(&mut Linker<Context>) -> anyhow::Result<()> + Send + Sync>,
    bytecode: &[u8],
    errors: &mut Vec<(EntityId, String)>,
) {
    let messenger = world.resource(messenger()).clone();
    let result = run_and_catch_panics(|| {
        ModuleState::new(
            bytecode,
            Box::new({
                let messenger = messenger.clone();
                move |world, msg| {
                    messenger(world, module_id, MessageType::Stdout, msg);
                }
            }),
            Box::new(move |world, msg| {
                messenger(world, module_id, MessageType::Stderr, msg);
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
                module_id,
                sms.clone(),
                &RunContext::new(world, "core/module_load", Entity::new()),
            ));

            world
                .add_component(module_id, state_component, sms)
                .unwrap();
        }
        Err(err) => errors.push((module_id, err)),
    }
}

pub fn unload<
    Bindings: Send + Sync + 'static,
    Context: WasmContext<Bindings> + Send + Sync + 'static,
    HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
>(
    world: &mut World,
    state_component: Component<ModuleState<Bindings, Context, HostGuestState>>,
    module_id: EntityId,
    reason: &str,
) -> Vec<(EntityId, String)> {
    let Ok(sms) = world.get_cloned(module_id, state_component) else { return vec![]; };

    let errors = run(
        world,
        state_component,
        module_id,
        sms,
        &RunContext::new(world, "core/module_unload", Entity::new()),
    )
    .into_iter()
    .collect_vec();

    let spawned_entities = world
        .get_mut(module_id, state_component)
        .map(|sms| std::mem::take(&mut sms.shared_state().write().base_mut().spawned_entities))
        .unwrap_or_default();

    if let Ok(module_errors) = world.get_mut(module_id, module_errors()) {
        module_errors.runtime.clear();
    }

    world.remove_component(module_id, state_component).unwrap();

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

pub fn update_errors<
    Bindings: Send + Sync + 'static,
    Context: WasmContext<Bindings> + Send + Sync + 'static,
    HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
>(
    world: &mut World,
    state_component: Component<ModuleState<Bindings, Context, HostGuestState>>,
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

        if let Ok(module_errors) = world.get_mut(*id, module_errors()) {
            let error_stream = match runtime {
                true => &mut module_errors.runtime,
                false => &mut module_errors.compiletime,
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
    state_component: Component<ModuleState<Bindings, Context, HostGuestState>>,
    id: EntityId,
    mut state: ModuleState<Bindings, Context, HostGuestState>,
    context: &RunContext,
) -> Option<(EntityId, String)> {
    profiling::scope!(
        "run",
        format!("{} - {}", get_module_name(world, id), context.event_name)
    );

    // If this is not a whitelisted event and it's not in the subscribed events,
    // skip over it
    if !["core/module_load", "core/frame"].contains(&context.event_name.as_str())
        && !state
            .shared_state
            .write()
            .base_mut()
            .subscribed_events
            .contains(&context.event_name)
    {
        return None;
    }

    let result = run_and_catch_panics(|| state.run(world, context));
    world.set(id, state_component, state).ok();

    result.err().map(|err| (id, err))
}

pub fn spawn_module(
    world: &mut World,
    name: &Identifier,
    description: String,
    enabled: bool,
) -> anyhow::Result<EntityId> {
    if query(())
        .incl(module())
        .iter(world, None)
        .any(|(id, _)| &get_module_name(world, id) == name)
    {
        anyhow::bail!("a WASM module by the name {name} already exists");
    }

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
