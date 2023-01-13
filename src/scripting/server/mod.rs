use std::{collections::HashMap, path::PathBuf, sync::Arc};

use elements_ecs::{components, EntityId, SystemGroup, World};
use elements_network::server::{ForkingEvent, ShutdownEvent};
use elements_scripting_host::{
    server::bindings::{Bindings, WasmServerContext},
    shared::{
        host_state::{spawn_script_module, HostState, MessageType},
        interface::get_scripting_interfaces,
        rustc::InstallDirs,
        util::get_module_name,
        BaseHostGuestState, File, ScriptModuleState,
    },
};

use crate::server::project_path;

pub type HostServerState = HostState<Bindings, WasmServerContext, BaseHostGuestState>;

pub type ScriptModuleServerState =
    ScriptModuleState<Bindings, WasmServerContext, BaseHostGuestState>;

components!("scripting::server", {
    // resource
    host_state: Arc<HostServerState>,
    // component
    script_module_state: ScriptModuleServerState,
});

pub fn init_all_components() {
    elements_scripting_host::server::init_components();
    init_components();
}

pub fn systems() -> SystemGroup {
    elements_scripting_host::server::systems(host_state(), false)
}

pub fn on_forking_systems() -> SystemGroup<ForkingEvent> {
    elements_scripting_host::server::on_forking_systems(host_state())
}

pub fn on_shutdown_systems() -> SystemGroup<ShutdownEvent> {
    elements_scripting_host::server::on_shutdown_systems(host_state())
}

pub async fn initialize(world: &mut World) -> anyhow::Result<()> {
    let rust_path = elements_std::path::normalize(&std::env::current_dir()?.join("rust"));

    let messenger = Arc::new(
        |world: &World, id: EntityId, type_: MessageType, message: &str| {
            let name = get_module_name(world, id);
            let (prefix, level) = match type_ {
                MessageType::Info => ("info", log::Level::Info),
                MessageType::Error => ("error", log::Level::Error),
                MessageType::Stdout => ("stdout", log::Level::Info),
                MessageType::Stderr => ("stderr", log::Level::Warn),
            };

            log::log!(
                level,
                "[{name}] {prefix}: {}",
                message.strip_suffix('\n').unwrap_or(message)
            );
        },
    );

    let project_path = world.resource(project_path()).clone();
    let new_host_state = HostServerState {
        messenger,
        scripting_interfaces: get_scripting_interfaces(),

        rust_path: rust_path.clone(),
        install_dirs: InstallDirs {
            rustup_path: rust_path.join("rustup"),
            cargo_path: rust_path.join("cargo"),
        },
        scripting_interface_root_path: project_path.join("interfaces"),
        templates_path: rust_path.join("templates"),
        workspace_path: project_path.clone(),
        scripts_path: project_path.join("scripts"),

        state_component: script_module_state(),
        make_wasm_context: Arc::new(|ctx, state| WasmServerContext::new(ctx, state)),
        add_to_linker: Arc::new(|linker| WasmServerContext::link(linker, |c| c)),

        _bindings: Default::default(),
    };
    new_host_state
        .initialize(world, "elements_scripting_interface")
        .await?;
    world.add_resource(host_state(), Arc::new(new_host_state));
    elements_scripting_host::server::initialize(world, host_state())?;

    let scripts_path = project_path.join("scripts");
    if scripts_path.exists() {
        for path in std::fs::read_dir(scripts_path)?
            .filter_map(Result::ok)
            .map(|de| de.path())
            .filter(|p| p.is_dir())
            .filter(|p| p.join("Cargo.toml").exists())
        {
            if let Some(file_name) = path.file_name() {
                let name = file_name.to_string_lossy();

                let files: HashMap<PathBuf, File> = walkdir::WalkDir::new(&path)
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter(|de| de.path().is_file())
                    .map(|de| {
                        Ok((
                            de.path().strip_prefix(&path)?.to_path_buf(),
                            File::new_at_now(std::fs::read_to_string(de.path())?),
                        ))
                    })
                    .collect::<anyhow::Result<_>>()?;

                spawn_script_module(
                    world,
                    name.as_ref(),
                    String::new(),
                    true,
                    files,
                    Default::default(),
                    Default::default(),
                )?;
            }
        }
    }

    Ok(())
}
