use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Context;
use elements_ecs::{
    query, uid, uid_lookup, Component, ComponentUnit as CU, EntityData, EntityId, World,
};
use elements_network::player::player;
use itertools::Itertools;
use parking_lot::Mutex;
use wasi_common::WasiCtx;

use crate::shared::{
    get_module_name, interface::write_scripting_interfaces, sanitize, script_module,
    script_module_bytecode, script_module_errors, scripting_interface_name,
    write_files_to_directory, FileMap, GetBaseHostGuestState, ParametersMap, ScriptContext,
    ScriptModule, ScriptModuleBytecode, ScriptModuleState, WasmContext,
};

use super::rustc::{self, InstallDirs};

pub const PARAMETER_CHANGE_DEBOUNCE_SECONDS: u64 = 2;
pub const MINIMUM_RUST_VERSION: (u32, u32, u32) = (1, 65, 0);
pub const MAXIMUM_ERROR_COUNT: usize = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessageType {
    Info,
    Error,
    Stdout,
    Stderr,
}

/// All paths specified should be absolute
pub struct HostState<
    Bindings: Send + Sync + 'static,
    Context: WasmContext<Bindings> + Send + Sync + 'static,
    HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
> {
    pub messenger: Arc<dyn Fn(&World, EntityId, MessageType, &str) + Send + Sync>,
    pub scripting_interfaces: HashMap<String, Vec<(PathBuf, String)>>,

    /// Where Rust should be installed
    pub rust_path: PathBuf,
    /// Where the Rust applications are installed. Should be underneath [rust_path].
    pub install_dirs: InstallDirs,
    /// Where the scripting interfaces should be installed, not the path to the scripting interface itself
    ///
    /// e.g. world/, not world/scripting_interface
    pub scripting_interface_root_path: PathBuf,
    /// Where the scripting templates should be stored
    pub templates_path: PathBuf,
    /// Where the root Cargo.toml for your scripts are
    pub workspace_path: PathBuf,
    /// Where the scripts are located
    pub scripts_path: PathBuf,

    pub state_component: Component<ScriptModuleState<Bindings, Context, HostGuestState>>,
    pub make_wasm_context:
        Arc<dyn Fn(WasiCtx, Arc<Mutex<HostGuestState>>) -> Context + Send + Sync>,
    pub add_to_linker:
        Arc<dyn Fn(&mut wasmtime::Linker<Context>) -> anyhow::Result<()> + Send + Sync>,

    pub _bindings: PhantomData<Bindings>,
}

impl<
        Bindings: Send + Sync + 'static,
        Context: WasmContext<Bindings> + Send + Sync + 'static,
        HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
    > Clone for HostState<Bindings, Context, HostGuestState>
{
    fn clone(&self) -> Self {
        Self {
            messenger: self.messenger.clone(),
            scripting_interfaces: self.scripting_interfaces.clone(),

            rust_path: self.rust_path.clone(),
            install_dirs: self.install_dirs.clone(),
            scripting_interface_root_path: self.scripting_interface_root_path.clone(),
            templates_path: self.templates_path.clone(),
            workspace_path: self.workspace_path.clone(),
            scripts_path: self.scripts_path.clone(),

            state_component: self.state_component.clone(),
            make_wasm_context: self.make_wasm_context.clone(),
            add_to_linker: self.add_to_linker.clone(),

            _bindings: self._bindings.clone(),
        }
    }
}

impl<
        Bindings: Send + Sync + 'static,
        Context: WasmContext<Bindings> + Send + Sync + 'static,
        HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
    > HostState<Bindings, Context, HostGuestState>
{
    pub async fn initialize(
        &self,
        world: &mut World,
        primary_scripting_interface_name: &str,
    ) -> anyhow::Result<()> {
        assert!(self
            .scripting_interfaces
            .contains_key(primary_scripting_interface_name));
        assert!([
            &self.rust_path,
            &self.install_dirs.cargo_path,
            &self.install_dirs.rustup_path,
            &self.scripting_interface_root_path,
            &self.templates_path,
            &self.workspace_path
        ]
        .iter()
        .all(|p| p.is_absolute()));

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
        world.add_resource(
            scripting_interface_name(),
            primary_scripting_interface_name.to_owned(),
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

        let target_dir = self.workspace_path.join("target");
        if !target_dir
            .join("wasm32-wasi")
            .join("release")
            .join("dummy.wasm")
            .exists()
        {
            log::info!("world does not have compiled scripts, copying precompiled template");
            std::fs::create_dir_all(&target_dir)
                .context("failed to create target directory for world")?;
            fs_extra::dir::copy(
                self.templates_path.join("scripts").join("target"),
                &self.workspace_path,
                &fs_extra::dir::CopyOptions {
                    overwrite: true,
                    ..Default::default()
                },
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
                    world.get_cloned(script_id, self.state_component).ok()?,
                    &ScriptContext::new(
                        world,
                        "core/player_leave",
                        vec![CU::new(elements_ecs::id(), *player_id)].into(),
                    ),
                )
            })
            .collect_vec();

        let spawned_entities = world
            .get_mut(script_id, self.state_component)
            .map(|sms| std::mem::take(&mut sms.shared_state().lock().base_mut().spawned_entities))
            .unwrap_or_default();

        if let Ok(script_module_errors) = world.get_mut(script_id, script_module_errors()) {
            script_module_errors.runtime.clear();
        }

        world
            .remove_component(script_id, self.state_component)
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
        let errors: Vec<(EntityId, String)> = query(self.state_component)
            .collect_cloned(world, None)
            .into_iter()
            .flat_map(|(id, sms)| self.run_script(world, id, sms, context))
            .collect();
        self.update_errors(world, &errors, true);
    }

    pub fn reload(&self, world: &mut World, scripts: &[(EntityId, Option<ScriptModuleBytecode>)]) {
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
                            .add_component(script_id, self.state_component, sms)
                            .unwrap();
                    }
                    Err(err) => errors.push((script_id, err)),
                }
            }

            self.update_errors(world, &errors, true);
        }
    }

    pub fn update_errors(&self, world: &mut World, errors: &[(EntityId, String)], runtime: bool) {
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
}

impl<
        Bindings: Send + Sync + 'static,
        Context: WasmContext<Bindings> + Send + Sync + 'static,
        HostGuestState: Default + GetBaseHostGuestState + Send + Sync + 'static,
    > HostState<Bindings, Context, HostGuestState>
{
    fn run_script(
        &self,
        world: &mut World,
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
        world.set(id, self.state_component, state).ok();

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

pub fn compile_module(
    sm: &ScriptModule,
    install_dirs: InstallDirs,
    workspace_path: PathBuf,
    scripts_path: PathBuf,
    name: String,
) -> Option<std::thread::JoinHandle<anyhow::Result<Vec<u8>>>> {
    let mut files = sm.files().clone();

    if let Some(file) = files.get_mut(Path::new("src/lib.rs")) {
        // HACK(mithun): figure out how to insert this without exposing it to the user
        if !file.contents.contains("fn call_main") {
            file.contents += indoc::indoc! {r#"

                #[no_mangle]
                pub extern "C" fn call_main(runtime_interface_version: u32) {
                    if INTERFACE_VERSION != runtime_interface_version {
                        panic!("This script was compiled with interface version {{INTERFACE_VERSION}}, but the script host is running with version {{runtime_interface_version}}");
                    }
                    run_async(main());
                }
            "#};
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
    }

    Some(std::thread::spawn(move || {
        rustc::build_module_in_workspace(&install_dirs, &workspace_path, &name)
    }))
}

fn build_script_template(
    install_dirs: &InstallDirs,
    template_path: &Path,
    scripting_interfaces: &HashMap<String, Vec<(PathBuf, String)>>,
    primary_scripting_interface_name: &str,
) -> Result<(), anyhow::Error> {
    let _ = std::fs::remove_dir_all(template_path);
    std::fs::create_dir_all(template_path)?;

    write_scripting_interfaces(scripting_interfaces, &template_path.join("interfaces"))
        .context("failed to write scripting interface for template")?;

    let dummy_name = "dummy";

    let scripts_path = template_path.join("scripts");
    super::write_workspace_files(&scripts_path, &[dummy_name.to_string()], true);

    let dummy_module = ScriptModule::new(
        dummy_name,
        "Dummy module",
        Default::default(),
        Default::default(),
        Default::default(),
        true,
        primary_scripting_interface_name,
    );
    let _dummy_bytecode = compile_module(
        &dummy_module,
        install_dirs.clone(),
        template_path.to_owned(),
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
