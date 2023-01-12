use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Context;
use elements_ecs::{
    components, query, uid, uid_lookup, Component, ComponentRegistry, ComponentUnit as CU,
    EntityData, EntityId, FnSystem, SystemGroup, World,
};
use elements_network::{
    player::player,
    server::{ForkingEvent, ShutdownEvent},
};
use elements_physics::{collider_loads, collisions, PxShapeUserData};
use itertools::Itertools;
use parking_lot::Mutex;
use physxx::{PxRigidActor, PxRigidActorRef, PxUserData};
use wasi_common::WasiCtx;

use crate::shared::{
    get_module_name,
    interface::write_scripting_interfaces,
    sanitize, script_module, script_module_bytecode, script_module_compiled, script_module_errors,
    scripting_interface_name, update_components,
    wasm::{GuestExports, WasmContext},
    write_files_to_directory, FileMap, GetBaseHostGuestState, ParametersMap, ScriptContext,
    ScriptModule, ScriptModuleBytecode, ScriptModuleErrors, ScriptModuleState,
};

use self::rustc::InstallDirs;

pub mod bindings;
pub mod implementation;
pub mod rustc;
mod wasm;

pub const PARAMETER_CHANGE_DEBOUNCE_SECONDS: u64 = 2;
pub const MINIMUM_RUST_VERSION: (u32, u32, u32) = (1, 65, 0);
pub const MAXIMUM_ERROR_COUNT: usize = 10;

components!("scripting::server", {
    // resources
    deferred_compilation_tasks: HashMap<EntityId, Instant>,
    compilation_tasks: HashMap<EntityId, Arc<std::thread::JoinHandle<anyhow::Result<Vec<u8>>>>>,
    docs_path: PathBuf,
});

/// The [host_state_component] resource *must* be initialized before this is called
pub fn systems<
    Bindings: Send + Sync + 'static,
    Context: WasmContext<Bindings> + Send + Sync + 'static,
    Exports: GuestExports<Bindings, Context> + Send + Sync + 'static,
    HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
>(
    host_state_component: Component<Arc<HostState<Bindings, Context, Exports, HostGuestState>>>,
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
                        .remove_component(id, host_state.server_state_component)
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
                    crate::shared::write_workspace_files(&host_state.scripts_path, &members);
                    crate::shared::remove_old_script_modules(&host_state.scripts_path, &members);
                }

                let tasks = ready_ids
                    .into_iter()
                    .filter_map(|id| {
                        let script_module = world.get_ref(id, script_module()).ok()?;

                        Some((
                            id,
                            Arc::new(compile_module_raw(
                                script_module,
                                host_state.install_dirs.clone(),
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
                let collisions = world.resource(collisions()).lock().clone();
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
                let host_state = host_state(world);
                let collider_loads = world.resource(collider_loads()).clone();
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
    Bindings: Send + Sync + 'static,
    Context: WasmContext<Bindings> + Send + Sync + 'static,
    Exports: GuestExports<Bindings, Context> + Send + Sync + 'static,
    HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
>(
    host_state_component: Component<Arc<HostState<Bindings, Context, Exports, HostGuestState>>>,
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
    Bindings: Send + Sync + 'static,
    Context: WasmContext<Bindings> + Send + Sync + 'static,
    Exports: GuestExports<Bindings, Context> + Send + Sync + 'static,
    HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
>(
    host_state_component: Component<Arc<HostState<Bindings, Context, Exports, HostGuestState>>>,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessageType {
    Info,
    Error,
    Stdout,
    Stderr,
}

pub struct HostState<
    Bindings: Send + Sync + 'static,
    Context: WasmContext<Bindings> + Send + Sync + 'static,
    Exports: GuestExports<Bindings, Context> + Send + Sync + 'static,
    HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
> {
    pub messenger: Arc<dyn Fn(&World, EntityId, MessageType, &str) + Send + Sync>,
    pub scripting_interfaces: HashMap<String, Vec<(PathBuf, String)>>,

    pub rust_path: PathBuf,
    pub install_dirs: InstallDirs,
    /// Where the scripting interfaces should be installed, not the path to the scripting interface itself
    ///
    /// e.g. world/, not world/scripting_interface
    pub scripting_interface_root_path: PathBuf,
    pub templates_path: PathBuf,
    pub scripts_path: PathBuf,

    pub server_state_component:
        Component<ScriptModuleState<Bindings, Context, Exports, HostGuestState>>,
    pub make_wasm_context:
        Arc<dyn Fn(WasiCtx, Arc<Mutex<HostGuestState>>) -> Context + Send + Sync>,
    pub add_to_linker:
        Arc<dyn Fn(&mut wasmtime::Linker<Context>) -> anyhow::Result<()> + Send + Sync>,

    pub _bindings: PhantomData<Bindings>,
}

impl<
        Bindings: Send + Sync + 'static,
        Context: WasmContext<Bindings> + Send + Sync + 'static,
        Exports: GuestExports<Bindings, Context> + Send + Sync + 'static,
        HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
    > Clone for HostState<Bindings, Context, Exports, HostGuestState>
{
    fn clone(&self) -> Self {
        Self {
            messenger: self.messenger.clone(),
            scripting_interfaces: self.scripting_interfaces.clone(),

            rust_path: self.rust_path.clone(),
            install_dirs: self.install_dirs.clone(),
            scripting_interface_root_path: self.scripting_interface_root_path.clone(),
            templates_path: self.templates_path.clone(),
            scripts_path: self.scripts_path.clone(),

            server_state_component: self.server_state_component.clone(),
            make_wasm_context: self.make_wasm_context.clone(),
            add_to_linker: self.add_to_linker.clone(),

            _bindings: self._bindings.clone(),
        }
    }
}

impl<
        Bindings: Send + Sync + 'static,
        Context: WasmContext<Bindings> + Send + Sync + 'static,
        Exports: GuestExports<Bindings, Context> + Send + Sync + 'static,
        HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
    > HostState<Bindings, Context, Exports, HostGuestState>
{
    pub async fn initialize(
        &self,
        world: &mut World,
        primary_scripting_interface_name: &str,
    ) -> anyhow::Result<()> {
        assert!(self
            .scripting_interfaces
            .contains_key(primary_scripting_interface_name));

        if !self.rust_path.exists() {
            let rustup_init_path = Path::new("./rustup-init");
            let err = rustc::download_and_install(&self.install_dirs, rustup_init_path)
                .await
                .err();
            if let Some(err) = err {
                std::fs::remove_dir_all(&self.rust_path)?;
                std::fs::remove_file(rustup_init_path)?;
                return Err(err);
            }
        }

        // Update Rust if we're below our minimum supported Rust version.
        if rustc::get_installed_version(&self.install_dirs)
            .context("failed to get rustc version")?
            < MINIMUM_RUST_VERSION
        {
            rustc::update_rust(&self.install_dirs).context("failed to update rust")?;
        }

        write_scripting_interfaces(
            &self.scripting_interfaces,
            &self.scripting_interface_root_path,
        )?;
        world.add_resource(deferred_compilation_tasks(), HashMap::new());
        world.add_resource(compilation_tasks(), HashMap::new());
        world.add_resource(
            scripting_interface_name(),
            primary_scripting_interface_name.to_owned(),
        );
        world.add_resource(
            docs_path(),
            rustc::document_module(
                &self.install_dirs,
                &self
                    .scripting_interface_root_path
                    .join(primary_scripting_interface_name),
            )
            .context("failed to document scripting interface")?,
        );

        // To speed up compilation of new maps with this version, we precompile a dummy script using the
        // scripting interface. Its resulting target folder will then be copied into this project's
        // scripts folder when there is not already a target folder available.
        if !self.templates_path.exists() {
            log::info!("no precompiled template available, building");
            build_script_template(
                &self.install_dirs,
                &self.templates_path,
                &self.scripting_interfaces,
                primary_scripting_interface_name,
            )
            .context("failed to build script template")?;
            log::info!("finished building precompiled template");
        }

        let target_dir = self.scripts_path.join("target");
        if !target_dir.exists() {
            log::info!("world does not have compiled scripts, copying precompiled template");
            std::fs::create_dir_all(&target_dir)
                .context("failed to create target directory for world")?;
            fs_extra::dir::copy(
                self.templates_path.join("scripts").join("target"),
                &self.scripts_path,
                &fs_extra::dir::CopyOptions::new(),
            )
            .context("failed to copy scripts to target")?;
        }

        Ok(())
    }

    pub fn reload_all(&self, world: &mut World) {
        let scripts = query((script_module(), script_module_bytecode()))
            .iter(world, None)
            .map(|(id, (sm, bc))| (id, sm.enabled.then(|| bc.clone())))
            .collect_vec();
        self.reload(world, &scripts);
    }

    pub fn unload(
        &self,
        world: &mut World,
        script_id: EntityId,
        players: &[EntityId],
        reason: &str,
    ) -> Vec<(EntityId, String)> {
        // TODO: replace with explicit ModuleUnload/ModuleLoad events
        // Run PlayerLeave events for all players in the world for the module.
        let errors = players
            .iter()
            .filter_map(|player_id| {
                self.run_script(
                    world,
                    script_id,
                    world
                        .get_cloned(script_id, self.server_state_component)
                        .ok()?,
                    &ScriptContext::new(
                        world,
                        "core/player_leave",
                        vec![CU::new(elements_ecs::id(), *player_id)].into(),
                    ),
                )
            })
            .collect_vec();

        let spawned_entities = world
            .get_mut(script_id, self.server_state_component)
            .map(|sms| std::mem::take(&mut sms.shared_state().lock().base_mut().spawned_entities))
            .unwrap_or_default();

        if let Ok(script_module_errors) = world.get_mut(script_id, script_module_errors()) {
            script_module_errors.runtime.clear();
        }

        world
            .remove_component(script_id, self.server_state_component)
            .unwrap();

        for uid in spawned_entities {
            if let Ok(id) = world.resource(uid_lookup()).get(&uid) {
                world.despawn(id);
            }
        }

        (self.messenger)(
            world,
            script_id,
            MessageType::Info,
            &format!("Unloaded (reason: {reason})"),
        );

        errors
    }

    pub fn run_all(&self, world: &mut World, context: &ScriptContext) {
        let errors: Vec<(EntityId, String)> = query(self.server_state_component)
            .collect_cloned(world, None)
            .into_iter()
            .flat_map(|(id, sms)| self.run_script(world, id, sms, context))
            .collect();
        self.update_errors(world, &errors, true);
    }
}

impl<
        Bindings: Send + Sync + 'static,
        Context: WasmContext<Bindings> + Send + Sync + 'static,
        Exports: GuestExports<Bindings, Context> + Send + Sync + 'static,
        HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
    > HostState<Bindings, Context, Exports, HostGuestState>
{
    fn reload(&self, world: &mut World, scripts: &[(EntityId, Option<ScriptModuleBytecode>)]) {
        let players = query(player()).collect_ids(world, None);
        for (script_id, bytecode) in scripts {
            let script_id = *script_id;
            let mut errors = self.unload(world, script_id, &players, "reloading");

            if let Some(bytecode) = bytecode {
                let make_wasm_context = self.make_wasm_context.clone();
                let add_to_linker = self.add_to_linker.clone();
                let result = run_and_catch_errors(|| {
                    let messenger = self.messenger.clone();
                    ScriptModuleState::new(
                        &bytecode.0,
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
                        errors.extend(self.run_script(
                            world,
                            script_id,
                            sms.clone(),
                            &ScriptContext::new(world, "core/module_load", EntityData::new()),
                        ));

                        // Run the PlayerJoin event for all players to simulate the world being loaded
                        // in for the module.
                        errors.extend(players.iter().filter_map(|player_id| {
                            let script_context = ScriptContext::new(
                                world,
                                "core/player_join",
                                vec![CU::new(elements_ecs::id(), *player_id)].into(),
                            );
                            self.run_script(world, script_id, sms.clone(), &script_context)
                        }));

                        world
                            .add_component(script_id, self.server_state_component, sms)
                            .unwrap();
                    }
                    Err(err) => errors.push((script_id, err)),
                }
            }

            self.update_errors(world, &errors, true);
        }
    }

    fn update_errors(&self, world: &mut World, errors: &[(EntityId, String)], runtime: bool) {
        let players = query(player()).collect_ids(world, None);

        for (id, err) in errors {
            (self.messenger)(
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
                    self.unload(world, *id, &players, "too many errors");
                }
            }
        }
    }

    fn run_script(
        &self,
        world: &mut World,
        id: EntityId,
        mut state: ScriptModuleState<Bindings, Context, Exports, HostGuestState>,
        context: &ScriptContext,
    ) -> Option<(EntityId, String)> {
        profiling::scope!(
            "run_script",
            format!("{} - {}", get_module_name(world, id), context.event_name)
        );

        if context.event_name != "core/module_load"
            && !state
                .shared_state
                .lock()
                .base_mut()
                .event
                .subscribed_events
                .contains(&context.event_name)
        {
            return None;
        }

        let result = run_and_catch_errors(|| state.run(world, context));
        let events_to_run = std::mem::take(&mut state.shared_state.lock().base_mut().event.events);
        world.set(id, self.server_state_component, state).ok();

        let err = result.err().map(|err| (id, err));
        // TODO(mithun): come up with a more intelligent dispatch scheme than this
        // This can very easily result in an infinite loop.
        // Things to fix include:
        // - don't let a script trigger an event on itself
        // - don't let two scripts chat with each other indefinitely (shunt them to the next tick)
        // - don't do the event dispatch in this function and instead do it *after* initial
        //   execution of all scripts
        for (event_name, event_data) in events_to_run {
            self.run_all(world, &ScriptContext::new(world, &event_name, event_data));
        }

        err
    }
}

pub fn spawn_script_module(
    world: &mut World,
    name: &str,
    description: String,
    enabled: bool,
    files: FileMap,
    parameters: ParametersMap,
    external_component_ids: HashSet<String>,
) -> anyhow::Result<EntityId> {
    if query(())
        .incl(script_module())
        .iter(world, None)
        .any(|(id, _)| get_module_name(world, id) == name)
    {
        anyhow::bail!("a script module by the name {name} already exists");
    }

    let scripting_interface_name = world.resource(scripting_interface_name()).clone();
    let sm = ScriptModule::new(
        name,
        description,
        files,
        parameters,
        external_component_ids,
        enabled,
        &scripting_interface_name,
    );
    Ok(EntityData::new()
        .set(elements_core::name(), name.to_string())
        .set(uid(), elements_ecs::EntityUid::create())
        .set(script_module(), sm)
        .spawn(world))
}

fn compile_module_raw(
    sm: &ScriptModule,
    install_dirs: InstallDirs,
    scripts_path: PathBuf,
    name: String,
) -> Option<std::thread::JoinHandle<anyhow::Result<Vec<u8>>>> {
    let mut files = sm.files().clone();
    if let Some(lib) = files.get_mut(Path::new("src/lib.rs")) {
        lib.contents.push_str(r#"
        #[no_mangle]
        pub extern "C" fn call_main(runtime_interface_version: u32) {{
            if INTERFACE_VERSION != runtime_interface_version {{
                panic!("This script was compiled with interface version {{INTERFACE_VERSION}}, but the script host is running with version {{interface_version}}");
            }}
            run_async(main());
        }}
    "#);
    }

    // Remove the directory to ensure there aren't any old files left around
    let script_path = scripts_path.join(sanitize(&name));
    let _ = std::fs::remove_dir_all(&script_path);
    write_files_to_directory(
        &script_path,
        &files
            .iter()
            .map(|(p, f)| (p.clone(), f.contents.clone()))
            .collect_vec(),
    )
    .unwrap();

    Some(std::thread::spawn(move || {
        rustc::build_module_in_workspace(&install_dirs, &scripts_path, &name)
    }))
}

fn build_script_template(
    install_dirs: &InstallDirs,
    templates_path: &Path,
    scripting_interfaces: &HashMap<String, Vec<(PathBuf, String)>>,
    primary_scripting_interface_name: &str,
) -> Result<(), anyhow::Error> {
    let _ = std::fs::remove_dir_all(templates_path);
    std::fs::create_dir_all(templates_path)?;

    write_scripting_interfaces(scripting_interfaces, templates_path)
        .context("failed to write scripting interface for template")?;

    let dummy_name = "dummy";

    let scripts_path = templates_path.join("scripts");
    super::shared::write_workspace_files(&scripts_path, &[dummy_name.to_string()]);

    let dummy_module = ScriptModule::new(
        dummy_name,
        "Dummy module",
        Default::default(),
        Default::default(),
        Default::default(),
        true,
        primary_scripting_interface_name,
    );
    let _dummy_bytecode = compile_module_raw(
        &dummy_module,
        install_dirs.clone(),
        scripts_path.clone(),
        dummy_name.to_owned(),
    )
    .context("failed to generate dummy compilation task")?
    .join()
    .unwrap()
    .context("failed to build dummy module")?;
    let _ = std::fs::remove_dir_all(scripts_path.join(dummy_name));

    Ok(())
}

fn run_and_catch_errors<R>(f: impl FnOnce() -> anyhow::Result<R>) -> Result<R, String> {
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
