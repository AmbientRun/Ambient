use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Instant};

use elements_ecs::{
    components, query, uid, Component, ComponentEntry, EntityData, EntityId, FnSystem, SystemGroup,
    World,
};
use elements_network::server::{ForkingEvent, ShutdownEvent};
use elements_physics::{collider_loads, collisions, PxShapeUserData};
use itertools::Itertools;
use parking_lot::RwLock;
use physxx::{PxRigidActor, PxRigidActorRef, PxUserData};
use wasi_common::WasiCtx;
use wasmtime::Linker;

use crate::shared::{
    compile, host_guest_state::GetBaseHostGuestState, interface::Host, messenger, reload,
    reload_all, run_all, rust_installation, script_module, script_module_bytecode,
    script_module_compiled, script_module_enabled, script_module_errors, script_module_owned_files,
    script_module_path, scripting_interface_name, unload, update_errors, util::get_module_name,
    MessageType, ScriptContext, ScriptModuleBytecode, ScriptModuleState, WasmContext,
};

pub mod bindings;
pub(crate) mod implementation;

pub const MINIMUM_RUST_VERSION: (u32, u32, u32) = (1, 65, 0);
pub const MAXIMUM_ERROR_COUNT: usize = 10;

components!("scripting::server", {
    deferred_compilation_tasks: HashMap<EntityId, Instant>,
    compilation_tasks: HashMap<EntityId, Arc<std::thread::JoinHandle<anyhow::Result<Vec<u8>>>>>,
});

pub fn systems<
    Bindings: Send + Sync + Host + 'static,
    Context: WasmContext<Bindings> + Send + Sync + 'static,
    HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
>(
    state_component: Component<ScriptModuleState<Bindings, Context, HostGuestState>>,
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
        "elements/scripting/server",
        vec![
            // If the script module exists, but does not have bytecode, populate its initial files.
            // This is only necessary the first time it's created, which is why we exclude any entities
            // that have bytecode on them. This exclusion also prevents this system from triggering when
            // the world is forked.
            query(script_module())
                .excl(script_module_bytecode())
                .spawned()
                .to_system(|q, world, qs, _| {
                    profiling::scope!("script module spawn population");

                    let scripting_interface_name =
                        world.resource(scripting_interface_name()).clone();

                    let ids = q.iter(world, qs).map(|(id, _)| id).collect_vec();
                    for id in ids {
                        let name = get_module_name(world, id);
                        if let Ok(owned_files) = world.get_mut(id, script_module_owned_files()) {
                            owned_files.populate(&name, &scripting_interface_name);
                        }
                    }
                }),
            query((script_module().changed(), script_module_enabled().changed()))
                .optional_changed(script_module_owned_files())
                .to_system({
                    let make_wasm_context = make_wasm_context;
                    let add_to_linker = add_to_linker;
                    move |q, world, qs, _| {
                        profiling::scope!("script module changed");
                        // Script module (files/enabled) changed, issue compilation tasks.
                        let mut tasks = vec![];
                        let mut to_disable = vec![];
                        let now = Instant::now();
                        for (id, (_, enabled)) in q.iter(world, qs) {
                            if *enabled {
                                tasks.push((id, now));
                            } else {
                                to_disable.push(id);
                            }
                        }

                        world
                            .resource_mut(deferred_compilation_tasks())
                            .extend(tasks);

                        for id in to_disable {
                            reload(
                                world,
                                state_component,
                                make_wasm_context(world),
                                add_to_linker(world),
                                &[(id, None)],
                            );
                            world.remove_component(id, state_component).unwrap();
                        }
                    }
                }),
            Box::new(FnSystem::new(move |world, _| {
                profiling::scope!("script module compilation deferred execution");
                // run deferred compilation tasks if they're ready to go
                let ready_ids = {
                    let deferred_tasks = world.resource_mut(deferred_compilation_tasks());
                    let ready_ids = deferred_tasks
                        .iter()
                        .filter(|(_, i)| Instant::now() > **i)
                        .map(|(id, _)| *id)
                        .collect_vec();
                    for id in &ready_ids {
                        deferred_tasks.remove(id);
                    }
                    ready_ids
                };

                let installation = world.resource(rust_installation());
                let tasks = ready_ids
                    .into_iter()
                    .map(|id| {
                        let owned_files = world.get_ref(id, script_module_owned_files()).ok();
                        let script_path = world.get_ref(id, script_module_path())?;

                        anyhow::Ok((
                            id,
                            Arc::new(compile(
                                installation.clone(),
                                script_path.clone(),
                                get_module_name(world, id).to_string(),
                                owned_files,
                            )),
                        ))
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();
                world.resource_mut(compilation_tasks()).extend(tasks);
            })),
            query((
                script_module(),
                script_module_bytecode().changed(),
                script_module_enabled(),
            ))
            .to_system(move |q, world, qs, _| {
                profiling::scope!("script module wasm recreation");
                let scripts = q
                    .iter(world, qs)
                    .map(|(id, (_, smb, enabled))| (id, enabled.then(|| smb.clone())))
                    .collect_vec();

                // Script module bytecode changed, recreate the WASM state
                reload(
                    world,
                    state_component,
                    make_wasm_context(world),
                    add_to_linker(world),
                    &scripts,
                );

                let messenger = world.resource(messenger()).clone();
                for (id, _) in &scripts {
                    (messenger)(world, *id, MessageType::Info, "Updated");
                }
            }),
            Box::new(FnSystem::new(move |world, _| {
                profiling::scope!("script module frame event");
                // trigger frame event
                run_all(
                    world,
                    state_component,
                    &ScriptContext::new(world, "core/frame", EntityData::new()),
                );
            })),
            Box::new(FnSystem::new(move |world, _| {
                profiling::scope!("script module collision event");
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
                        &ScriptContext::new(
                            world,
                            "core/collision",
                            vec![ComponentEntry::new(elements_ecs::ids(), ids)].into(),
                        ),
                    );
                }
            })),
            Box::new(FnSystem::new(move |world, _| {
                profiling::scope!("script module collider loads");
                // trigger collider loads
                let collider_loads = match world.resource_opt(collider_loads()) {
                    Some(collider_loads) => collider_loads.clone(),
                    None => return,
                };
                for id in collider_loads {
                    run_all(
                        world,
                        state_component,
                        &ScriptContext::new(
                            world,
                            "core/collider_load",
                            vec![ComponentEntry::new(elements_ecs::id(), id)].into(),
                        ),
                    );
                }
            })),
            query(uid()).spawned().to_system(move |q, world, qs, _| {
                for (id, uid) in q.collect_cloned(world, qs) {
                    run_all(
                        world,
                        state_component,
                        &ScriptContext::new(
                            world,
                            "core/entity_spawn",
                            vec![
                                ComponentEntry::new(elements_ecs::id(), id),
                                ComponentEntry::new(elements_ecs::uid(), uid),
                            ]
                            .into(),
                        ),
                    );
                }
            }),
            Box::new(FnSystem::new(move |world, _| {
                profiling::scope!("script module process compilation tasks");
                // process all compilation tasks and store their result or report errors
                let mut successful_updates = vec![];
                let mut errors = vec![];

                {
                    let tasks = world.resource_mut(compilation_tasks());
                    let completed_tasks = tasks
                        .iter()
                        .filter(|(_, t)| t.is_finished() && Arc::strong_count(t) == 1)
                        .map(|(id, _)| *id)
                        .collect_vec();

                    for id in completed_tasks {
                        let task = match tasks.remove(&id) {
                            Some(task) => task,
                            None => continue,
                        };
                        let task = Arc::try_unwrap(task).unwrap();
                        let result = task.join();

                        match result.unwrap() {
                            Ok(bytecode) => successful_updates.push((id, bytecode)),
                            Err(err) => errors.push((id, err.to_string())),
                        };
                    }
                }

                for id in successful_updates
                    .iter()
                    .map(|t| t.0)
                    .chain(errors.iter().map(|t| t.0))
                {
                    world
                        .set(id, script_module_errors(), Default::default())
                        .unwrap();
                }

                for (id, bytecode) in successful_updates {
                    world
                        .add_components(
                            id,
                            EntityData::new()
                                .set(script_module_bytecode(), ScriptModuleBytecode(bytecode))
                                .set(script_module_compiled(), ()),
                        )
                        .unwrap();
                }
                update_errors(world, state_component, &errors, false);
            })),
        ],
    )
}

pub fn on_forking_systems<
    Bindings: Send + Sync + Host + 'static,
    Context: WasmContext<Bindings> + Send + Sync + 'static,
    HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
>(
    state_component: Component<ScriptModuleState<Bindings, Context, HostGuestState>>,
    make_wasm_context_component: Component<
        Arc<dyn Fn(WasiCtx, Arc<RwLock<HostGuestState>>) -> Context + Send + Sync>,
    >,
    add_to_linker_component: Component<
        Arc<dyn Fn(&mut Linker<Context>) -> anyhow::Result<()> + Send + Sync>,
    >,
) -> SystemGroup<ForkingEvent> {
    SystemGroup::new(
        "core/scripting/server/on_forking_systems",
        vec![Box::new(FnSystem::new(move |world, _| {
            let make_wasm_context = world.resource(make_wasm_context_component).clone();
            let add_to_linker = world.resource(add_to_linker_component).clone();

            // Reset the states of all the scripts when we fork.
            reload_all(world, state_component, make_wasm_context, add_to_linker);
        }))],
    )
}

pub fn on_shutdown_systems<
    Bindings: Send + Sync + Host + 'static,
    Context: WasmContext<Bindings> + Send + Sync + 'static,
    HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
>(
    state_component: Component<ScriptModuleState<Bindings, Context, HostGuestState>>,
) -> SystemGroup<ShutdownEvent> {
    SystemGroup::new(
        "core/scripting/server/on_shutdown_systems",
        vec![Box::new(FnSystem::new(move |world, _| {
            let scripts = query(()).incl(script_module()).collect_ids(world, None);
            for script_id in scripts {
                let errors = unload(world, state_component, script_id, "shutting down");
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
    scripting_interfaces: HashMap<String, Vec<(PathBuf, String)>>,

    primary_scripting_interface_name: &str,

    // Where Rust should be installed
    rust_path: PathBuf,
    // Where the scripting interfaces should be installed, not the path to the scripting interface itself
    //
    // e.g. world/, not world/scripting_interface
    scripting_interface_root_path: PathBuf,

    (make_wasm_context_component, make_wasm_context): (
        Component<Arc<dyn Fn(WasiCtx, Arc<RwLock<HostGuestState>>) -> Context + Send + Sync>>,
        Arc<dyn Fn(WasiCtx, Arc<RwLock<HostGuestState>>) -> Context + Send + Sync>,
    ),
    (add_to_linker_component, add_to_linker): (
        Component<Arc<dyn Fn(&mut Linker<Context>) -> anyhow::Result<()> + Send + Sync>>,
        Arc<dyn Fn(&mut Linker<Context>) -> anyhow::Result<()> + Send + Sync>,
    ),
) -> anyhow::Result<()> {
    super::shared::initialize(
        world,
        messenger,
        scripting_interfaces,
        primary_scripting_interface_name,
        rust_path,
        scripting_interface_root_path.clone(),
    )
    .await?;

    world.add_resource(deferred_compilation_tasks(), HashMap::new());
    world.add_resource(compilation_tasks(), HashMap::new());
    world.add_resource(make_wasm_context_component, make_wasm_context);
    world.add_resource(add_to_linker_component, add_to_linker);

    Ok(())
}
