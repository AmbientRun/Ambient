use std::sync::Arc;

use itertools::Itertools;
use kiwi_ecs::{
    query, Component, ComponentEntry, EntityData, EntityId, FnSystem, SystemGroup, World,
};
use kiwi_network::server::{ForkingEvent, ShutdownEvent};
use kiwi_physics::{collider_loads, collisions, PxShapeUserData};
use parking_lot::RwLock;
use physxx::{PxRigidActor, PxRigidActorRef, PxUserData};
use wasi_common::WasiCtx;
use wasmtime::Linker;

use crate::shared::{
    host_guest_state::GetBaseHostGuestState, interface::host::Host, module, module_bytecode,
    module_enabled, reload, reload_all, run_all, unload, update_errors, MessageType, ModuleState,
    RunContext, WasmContext,
};

pub mod bindings;
pub(crate) mod implementation;

pub const MAXIMUM_ERROR_COUNT: usize = 10;

pub fn systems<
    Bindings: Send + Sync + Host + 'static,
    Context: WasmContext<Bindings> + Send + Sync + 'static,
    HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
>(
    state_component: Component<ModuleState<Bindings, Context, HostGuestState>>,
    make_wasm_context_component: Component<
        Arc<dyn Fn(WasiCtx, Arc<RwLock<HostGuestState>>) -> Context + Send + Sync>,
    >,
    add_to_linker_component: Component<
        Arc<dyn Fn(&mut Linker<Context>) -> anyhow::Result<()> + Send + Sync>,
    >,
) -> SystemGroup {
    let make_wasm_context = move |w: &World| w.resource(make_wasm_context_component).clone();
    let add_to_linker = move |w: &World| w.resource(add_to_linker_component).clone();

    SystemGroup::new(
        "core/wasm/server",
        vec![
            query((module_bytecode(), module_enabled().changed())).to_system(
                move |q, world, qs, _| {
                    profiling::scope!("WASM module reloads");
                    let modules = q
                        .iter(world, qs)
                        .filter(|(id, (_, enabled))| {
                            let has_state = world.has_component(*id, state_component);
                            **enabled != has_state
                        })
                        .map(|(id, (bytecode, enabled))| (id, (bytecode.clone(), *enabled)))
                        .collect_vec();

                    for (id, (bytecode, enabled)) in modules {
                        reload(
                            world,
                            state_component,
                            make_wasm_context(world),
                            add_to_linker(world),
                            id,
                            enabled.then_some(bytecode),
                        );
                    }
                },
            ),
            Box::new(FnSystem::new(move |world, _| {
                profiling::scope!("WASM module frame event");
                // trigger frame event
                run_all(
                    world,
                    state_component,
                    &RunContext::new(world, "core/frame", EntityData::new()),
                );
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
                        state_component,
                        &RunContext::new(
                            world,
                            "core/collision",
                            vec![ComponentEntry::new(kiwi_ecs::ids(), ids)].into(),
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
                        state_component,
                        &RunContext::new(
                            world,
                            "core/collider_load",
                            vec![ComponentEntry::new(kiwi_ecs::id(), id)].into(),
                        ),
                    );
                }
            })),
            query(()).spawned().to_system(move |q, world, qs, _| {
                profiling::scope!("WASM module entity spawn");
                for (id, _) in q.collect_cloned(world, qs) {
                    run_all(
                        world,
                        state_component,
                        &RunContext::new(
                            world,
                            "core/entity_spawn",
                            vec![ComponentEntry::new(kiwi_ecs::id(), id)].into(),
                        ),
                    );
                }
            }),
        ],
    )
}

pub fn on_forking_systems<
    Bindings: Send + Sync + Host + 'static,
    Context: WasmContext<Bindings> + Send + Sync + 'static,
    HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
>(
    state_component: Component<ModuleState<Bindings, Context, HostGuestState>>,
    make_wasm_context_component: Component<
        Arc<dyn Fn(WasiCtx, Arc<RwLock<HostGuestState>>) -> Context + Send + Sync>,
    >,
    add_to_linker_component: Component<
        Arc<dyn Fn(&mut Linker<Context>) -> anyhow::Result<()> + Send + Sync>,
    >,
) -> SystemGroup<ForkingEvent> {
    SystemGroup::new(
        "core/wasm/server/on_forking_systems",
        vec![Box::new(FnSystem::new(move |world, _| {
            let make_wasm_context = world.resource(make_wasm_context_component).clone();
            let add_to_linker = world.resource(add_to_linker_component).clone();

            // Reset the states of all the modules when we fork.
            reload_all(world, state_component, make_wasm_context, add_to_linker);
        }))],
    )
}

pub fn on_shutdown_systems<
    Bindings: Send + Sync + Host + 'static,
    Context: WasmContext<Bindings> + Send + Sync + 'static,
    HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
>(
    state_component: Component<ModuleState<Bindings, Context, HostGuestState>>,
) -> SystemGroup<ShutdownEvent> {
    SystemGroup::new(
        "core/wasm/server/on_shutdown_systems",
        vec![Box::new(FnSystem::new(move |world, _| {
            let modules = query(()).incl(module()).collect_ids(world, None);
            for module_id in modules {
                let errors = unload(world, state_component, module_id, "shutting down");
                update_errors(world, state_component, &errors, true);
            }
        }))],
    )
}

#[allow(clippy::too_many_arguments)]
pub async fn initialize<
    Bindings: Send + Sync + Host + 'static,
    Context: WasmContext<Bindings> + Send + Sync + 'static,
    HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
>(
    world: &mut World,

    messenger: Arc<dyn Fn(&World, EntityId, MessageType, &str) + Send + Sync>,

    (make_wasm_context_component, make_wasm_context): (
        Component<Arc<dyn Fn(WasiCtx, Arc<RwLock<HostGuestState>>) -> Context + Send + Sync>>,
        Arc<dyn Fn(WasiCtx, Arc<RwLock<HostGuestState>>) -> Context + Send + Sync>,
    ),
    (add_to_linker_component, add_to_linker): (
        Component<Arc<dyn Fn(&mut Linker<Context>) -> anyhow::Result<()> + Send + Sync>>,
        Arc<dyn Fn(&mut Linker<Context>) -> anyhow::Result<()> + Send + Sync>,
    ),
) -> anyhow::Result<()> {
    super::shared::initialize(world, messenger).await?;
    world.add_resource(make_wasm_context_component, make_wasm_context);
    world.add_resource(add_to_linker_component, add_to_linker);

    Ok(())
}
