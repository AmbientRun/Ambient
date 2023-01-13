use std::{
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Context;
use elements_ecs::{
    components, query, uid, Component, ComponentRegistry, ComponentUnit as CU, EntityData,
    EntityId, FnSystem, SystemGroup, World,
};
use elements_network::{
    player::player,
    server::{ForkingEvent, ShutdownEvent},
};
use elements_physics::{collider_loads, collisions, PxShapeUserData};
use itertools::Itertools;

use physxx::{PxRigidActor, PxRigidActorRef, PxUserData};

use crate::shared::{
    get_module_name,
    host_state::{compile_module, HostState, MessageType},
    interface::Host,
    rustc, script_module, script_module_bytecode, script_module_compiled, script_module_errors,
    scripting_interface_name, update_components,
    wasm::WasmContext,
    GetBaseHostGuestState, ScriptContext, ScriptModuleBytecode, ScriptModuleErrors,
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

/// The [host_state_component] resource *must* be initialized before this is called
pub fn systems<
    Bindings: Send + Sync + Host + 'static,
    Context: WasmContext<Bindings> + Send + Sync + 'static,
    HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
>(
    host_state_component: Component<Arc<HostState<Bindings, Context, HostGuestState>>>,
    update_workspace_toml: bool,
) -> SystemGroup {
    // Update the scripts whenever the external components change.
    let (update_tx, update_rx) = flume::unbounded();
    ComponentRegistry::get_mut()
        .on_external_components_change
        .add(Arc::new(move || {
            update_tx.send(()).unwrap();
        }));

    let host_state = move |world: &World| world.resource(host_state_component).clone();

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
            query(script_module().changed()).to_system(move |q, world, qs, _| {
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

                let host_state = host_state(world);
                for id in to_disable {
                    host_state.reload(world, &[(id, None)]);
                    world
                        .remove_component(id, host_state.state_component)
                        .unwrap();
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

                let host_state = host_state(world);

                if !ready_ids.is_empty() {
                    // Write all workspace-related state to disk.
                    let members = crate::shared::all_module_names_sanitized(world, false);
                    crate::shared::write_workspace_files(
                        &host_state.workspace_path,
                        &members,
                        update_workspace_toml,
                    );
                    crate::shared::remove_old_script_modules(&host_state.scripts_path, &members);
                }

                let tasks = ready_ids
                    .into_iter()
                    .filter_map(|id| {
                        let script_module = world.get_ref(id, script_module()).ok()?;

                        Some((
                            id,
                            Arc::new(compile_module(
                                script_module,
                                host_state.install_dirs.clone(),
                                host_state.workspace_path.clone(),
                                host_state.scripts_path.clone(),
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
                    let host_state = host_state(world);
                    host_state.reload(world, &scripts);
                    for (id, _) in &scripts {
                        (host_state.messenger)(world, *id, MessageType::Info, "Updated");
                    }
                },
            ),
            query(player()).spawned().to_system(move |q, world, qs, _| {
                profiling::scope!("script module player join event");
                // trigger player join event
                let host_state = host_state(world);
                for (id, _) in q.collect_cloned(world, qs) {
                    host_state.run_all(
                        world,
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
                    let host_state = host_state(world);
                    for (id, _) in q.collect_cloned(world, qs) {
                        host_state.run_all(
                            world,
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
                let host_state = host_state(world);
                host_state.run_all(
                    world,
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
                let host_state = host_state(world);
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
                    host_state.run_all(
                        world,
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
                let host_state = host_state(world);
                for id in collider_loads {
                    host_state.run_all(
                        world,
                        &ScriptContext::new(
                            world,
                            "core/collider_load",
                            vec![CU::new(elements_ecs::id(), id)].into(),
                        ),
                    );
                }
            })),
            query(uid()).spawned().to_system(move |q, world, qs, _| {
                let host_state = host_state(world);
                for (id, uid) in q.collect_cloned(world, qs) {
                    host_state.run_all(
                        world,
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
                host_state(world).update_errors(world, &errors, false);
            })),
            Box::new(FnSystem::new(move |world, _| {
                if update_rx.drain().count() > 0 {
                    update_components(world);
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
    host_state_component: Component<Arc<HostState<Bindings, Context, HostGuestState>>>,
) -> SystemGroup<ForkingEvent> {
    SystemGroup::new(
        "core/scripting/server/on_forking_systems",
        vec![Box::new(FnSystem::new(move |world, _| {
            // Reset the states of all the scripts when we fork.
            world
                .resource(host_state_component)
                .clone()
                .reload_all(world);
        }))],
    )
}

pub fn on_shutdown_systems<
    Bindings: Send + Sync + Host + 'static,
    Context: WasmContext<Bindings> + Send + Sync + 'static,
    HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
>(
    host_state_component: Component<Arc<HostState<Bindings, Context, HostGuestState>>>,
) -> SystemGroup<ShutdownEvent> {
    SystemGroup::new(
        "core/scripting/server/on_shutdown_systems",
        vec![Box::new(FnSystem::new(move |world, _| {
            let scripts = query(()).incl(script_module()).collect_ids(world, None);
            let players = query(player()).collect_ids(world, None);
            let host_state = world.resource(host_state_component).clone();
            for script_id in scripts {
                let errors = host_state.unload(world, script_id, &players, "shutting down");
                host_state.update_errors(world, &errors, true);
            }
        }))],
    )
}

pub fn initialize<
    Bindings: Send + Sync + Host + 'static,
    Context: WasmContext<Bindings> + Send + Sync + 'static,
    HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
>(
    world: &mut World,
    host_state_component: Component<Arc<HostState<Bindings, Context, HostGuestState>>>,
) -> anyhow::Result<()> {
    let install_dirs = world.resource(host_state_component).install_dirs.clone();
    let scripting_interface_root_path = world
        .resource(host_state_component)
        .scripting_interface_root_path
        .clone();
    let primary_scripting_interface_name = world.resource(scripting_interface_name()).clone();

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

    Ok(())
}
