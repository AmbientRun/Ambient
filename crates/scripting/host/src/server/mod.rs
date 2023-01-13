use std::{
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Context;
use elements_core::name;
use elements_ecs::{
    components, query, query_mut, uid, Component, ComponentRegistry, ComponentUnit as CU,
    EntityData, EntityId, FnSystem, SystemGroup, World,
};
use elements_network::{
    player::player,
    server::{ForkingEvent, ShutdownEvent},
};
use elements_physics::{collider_loads, collisions, PxShapeUserData};
use itertools::Itertools;

use parking_lot::RwLock;
use physxx::{PxRigidActor, PxRigidActorRef, PxUserData};
use wasi_common::WasiCtx;
use wasmtime::Linker;

use crate::shared::{
    compile,
    host_guest_state::GetBaseHostGuestState,
    install_dirs,
    interface::Host,
    messenger, reload, reload_all, run_all, rustc, script_module, script_module_bytecode,
    script_module_compiled, script_module_errors, scripting_interface_name, scripts_path, unload,
    update_errors,
    util::{
        all_module_names_sanitized, get_module_name, remove_old_script_modules,
        write_workspace_files,
    },
    workspace_path, MessageType, ScriptContext, ScriptModuleBytecode, ScriptModuleErrors,
    ScriptModuleState, WasmContext,
};

pub mod bindings;
pub mod implementation;

pub const PARAMETER_CHANGE_DEBOUNCE_SECONDS: u64 = 2;
pub const MINIMUM_RUST_VERSION: (u32, u32, u32) = (1, 65, 0);
pub const MAXIMUM_ERROR_COUNT: usize = 10;

components!("scripting::server", {
    deferred_compilation_tasks: HashMap<EntityId, Instant>,
    compilation_tasks: HashMap<EntityId, Arc<std::thread::JoinHandle<anyhow::Result<Vec<u8>>>>>,
    docs_path: PathBuf,
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
    update_workspace_toml: bool,
) -> SystemGroup {
    // Update the scripts whenever the external components change.
    let (update_tx, update_rx) = flume::unbounded();
    ComponentRegistry::get_mut()
        .on_external_components_change
        .add(Arc::new(move || {
            update_tx.send(()).unwrap();
        }));

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
                        world
                            .get_mut(id, script_module())
                            .unwrap()
                            .populate_files(&name, &scripting_interface_name);

                        world
                            .add_component(
                                id,
                                script_module_errors(),
                                ScriptModuleErrors::default(),
                            )
                            .unwrap();
                    }
                }),
            query(script_module().changed()).to_system({
                let make_wasm_context = make_wasm_context;
                let add_to_linker = add_to_linker;
                move |q, world, qs, _| {
                    profiling::scope!("script module changed");
                    // Script module (files/enabled) changed, issue compilation tasks.
                    // If the last edit was a parameter, defer the compilation so that users can
                    // edit parameters without forcing a compilation task to be issued on each keystroke.
                    let mut tasks = vec![];
                    let mut to_disable = vec![];
                    let now = Instant::now();
                    for (id, sm) in q.iter(world, qs) {
                        match (sm.enabled, sm.last_updated_by_parameters()) {
                            (true, true) => tasks.push((
                                id,
                                now + Duration::from_secs(PARAMETER_CHANGE_DEBOUNCE_SECONDS),
                            )),
                            (true, false) => tasks.push((id, now)),
                            (false, _) => {
                                to_disable.push(id);
                            }
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

                let workspace_path = world.resource(workspace_path()).clone();
                let scripts_path = world.resource(scripts_path()).clone();

                if !ready_ids.is_empty() {
                    // Write all workspace-related state to disk.
                    let members = all_module_names_sanitized(world, false);
                    write_workspace_files(&workspace_path, &members, update_workspace_toml);
                    remove_old_script_modules(&scripts_path, &members);
                }

                let install_dirs = world.resource(install_dirs()).clone();
                let tasks = ready_ids
                    .into_iter()
                    .filter_map(|id| {
                        let script_module = world.get_ref(id, script_module()).ok()?;

                        Some((
                            id,
                            Arc::new(compile(
                                script_module,
                                install_dirs.clone(),
                                workspace_path.clone(),
                                scripts_path.clone(),
                                get_module_name(world, id),
                            )?),
                        ))
                    })
                    .collect_vec();
                world.resource_mut(compilation_tasks()).extend(tasks);
            })),
            query((script_module(), script_module_bytecode().changed())).to_system(
                move |q, world, qs, _| {
                    profiling::scope!("script module wasm recreation");
                    let scripts = q
                        .iter(world, qs)
                        .map(|(id, (sm, smb))| (id, sm.enabled.then(|| smb.clone())))
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
                },
            ),
            query(player()).spawned().to_system(move |q, world, qs, _| {
                profiling::scope!("script module player join event");
                // trigger player join event
                for (id, _) in q.collect_cloned(world, qs) {
                    run_all(
                        world,
                        state_component,
                        &ScriptContext::new(
                            world,
                            "core/player_join",
                            vec![CU::new(elements_ecs::id(), id)].into(),
                        ),
                    );
                }
            }),
            query(player())
                .despawned()
                .to_system(move |q, world, qs, _| {
                    profiling::scope!("script module player leave event");
                    // trigger player leave event
                    for (id, _) in q.collect_cloned(world, qs) {
                        run_all(
                            world,
                            state_component,
                            &ScriptContext::new(
                                world,
                                "core/player_leave",
                                vec![CU::new(elements_ecs::id(), id)].into(),
                            ),
                        );
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
                            vec![CU::new(elements_ecs::ids(), ids)].into(),
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
                            vec![CU::new(elements_ecs::id(), id)].into(),
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
                                CU::new(elements_ecs::id(), id),
                                CU::new(elements_ecs::uid(), uid),
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
            Box::new(FnSystem::new(move |world, _| {
                if update_rx.drain().count() == 0 {
                    return;
                }

                let scripting_interface_name = world.resource(scripting_interface_name()).clone();
                for (_, sm, name) in query_mut(script_module(), name()).iter(world, None) {
                    sm.populate_files(name, &scripting_interface_name);
                }
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
            let players = query(player()).collect_ids(world, None);
            for script_id in scripts {
                let errors = unload(world, state_component, script_id, &players, "shutting down");
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
    // Where the scripting templates should be stored
    templates_path: PathBuf,
    // Where the root Cargo.toml for your scripts are
    workspace_path: PathBuf,
    // Where the scripts are located
    scripts_path: PathBuf,

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
        templates_path,
        workspace_path,
        scripts_path,
    )
    .await?;

    let install_dirs = world.resource(super::shared::install_dirs()).clone();

    world.add_resource(deferred_compilation_tasks(), HashMap::new());
    world.add_resource(compilation_tasks(), HashMap::new());
    world.add_resource(
        docs_path(),
        rustc::document_module(
            &install_dirs,
            &scripting_interface_root_path.join(primary_scripting_interface_name),
        )
        .context("failed to document scripting interface")?,
    );
    world.add_resource(make_wasm_context_component, make_wasm_context);
    world.add_resource(add_to_linker_component, add_to_linker);

    Ok(())
}
