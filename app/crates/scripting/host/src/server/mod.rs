use std::{path::PathBuf, sync::Arc};

use elements_ecs::{components, EntityId, SystemGroup, World};
use elements_network::server::{ForkingEvent, ShutdownEvent};
use elements_scripting_host::{
    server::bindings::{
        Bindings as ElementsBindings, WasmServerContext as ElementsWasmServerContext,
    },
    shared::{
        host_guest_state::BaseHostGuestState, spawn_script, util::get_module_name, BaseWasmContext,
        MessageType, ScriptModuleState, WasmContext,
    },
    Linker, WasiCtx,
};
use parking_lot::RwLock;

use self::{
    bindings::Bindings,
    interface::{get_scripting_interfaces, SCRIPTING_INTERFACE_NAME},
};

mod bindings;
mod implementation;
mod interface;

pub type ScriptModuleServerState =
    ScriptModuleState<ElementsBindings, WasmServerContext, BaseHostGuestState>;

components!("scripting::server", {
    // component
    script_module_state: ScriptModuleServerState,
    // resource
    make_wasm_context: Arc<dyn Fn(WasiCtx, Arc<RwLock<BaseHostGuestState>>) -> WasmServerContext + Send + Sync>,
    add_to_linker: Arc<dyn Fn(&mut Linker<WasmServerContext>) -> anyhow::Result<()> + Send + Sync>,
});

pub struct WasmServerContext {
    pub elements_context: ElementsWasmServerContext,
    pub dims_bindings: Bindings,
}
impl WasmServerContext {
    pub fn new(wasi: WasiCtx, shared_state: Arc<RwLock<BaseHostGuestState>>) -> Self {
        Self {
            elements_context: ElementsWasmServerContext::new(wasi, shared_state.clone()),
            dims_bindings: Bindings::new(shared_state),
        }
    }

    pub fn link(linker: &mut Linker<WasmServerContext>) -> anyhow::Result<()> {
        ElementsWasmServerContext::link(linker, |cx| &mut cx.elements_context)?;
        interface::host::add_to_linker(linker, |cx| &mut cx.dims_bindings)
    }
}
impl WasmContext<ElementsBindings> for WasmServerContext {
    fn base_wasm_context_mut(&mut self) -> &mut BaseWasmContext {
        self.elements_context.base_wasm_context_mut()
    }
}

pub fn init_all_components() {
    elements_scripting_host::server::init_components();
    init_components();
}

pub fn systems() -> SystemGroup {
    elements_scripting_host::server::systems(
        script_module_state(),
        make_wasm_context(),
        add_to_linker(),
    )
}

pub fn on_forking_systems() -> SystemGroup<ForkingEvent> {
    elements_scripting_host::server::on_forking_systems(
        script_module_state(),
        make_wasm_context(),
        add_to_linker(),
    )
}

pub fn on_shutdown_systems() -> SystemGroup<ShutdownEvent> {
    elements_scripting_host::server::on_shutdown_systems(script_module_state())
}

pub async fn initialize(
    world: &mut World,
    project_path: PathBuf,
    manifest: &elements_project::Manifest,
) -> anyhow::Result<()> {
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

    elements_scripting_host::server::initialize(
        world,
        messenger,
        get_scripting_interfaces(),
        SCRIPTING_INTERFACE_NAME,
        rust_path.clone(),
        project_path.join("interfaces"),
        (
            make_wasm_context(),
            Arc::new(|ctx, state| WasmServerContext::new(ctx, state)),
        ),
        (
            add_to_linker(),
            Arc::new(|linker| WasmServerContext::link(linker)),
        ),
    )
    .await?;

    let cargo_toml_path = project_path.join("Cargo.toml");
    if cargo_toml_path.exists() {
        let toml = cargo_toml::Manifest::from_str(&std::fs::read_to_string(&cargo_toml_path)?)?;

        match toml.package {
            Some(package) if package.name == manifest.project.id.as_ref() => {}
            Some(package) => {
                anyhow::bail!("The name of the package in the Cargo.toml ({}) does not match the project's ID ({})", package.name, manifest.project.id);
            }
            None => anyhow::bail!(
                "No [package] present in Cargo.toml for project {}",
                manifest.project.id.as_ref()
            ),
        }

        spawn_script(
            world,
            &manifest.project.id,
            manifest.project.description.clone().unwrap_or_default(),
            true,
            project_path,
            None,
        )?;
    }

    Ok(())
}
