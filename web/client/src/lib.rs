use std::sync::OnceLock;

use ambient_app::App;
use ambient_cameras::UICamera;
use ambient_core::{
    camera::active_camera,
    window::{ExitStatus, WindowCtl},
};
use ambient_ui_native::{
    element::{ElementComponentExt, Group},
    padding, Borders, WindowSized,
};
use app::MainApp;
use tracing_subscriber::{filter::LevelFilter, prelude::*, registry};
use tracing_web::MakeConsoleWriter;
use wasm_bindgen::prelude::*;

mod app;
mod wasm;

static APP_CONTROL: OnceLock<flume::Sender<WindowCtl>> = OnceLock::new();

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
/// TODO: make private if no longer needed by the dioxus example?
/// This is inteded to be used by a loosely typed settings object from javascript allow for
/// backwards and forwards compatibility when new fields are added to the settings object in a
/// "best fit" approach, similar to the `settings` in the LanguageServerProtocol spec.
pub struct Settings {
    pub enable_logging: bool,
    pub enable_panic_hook: bool,
}

/// Initialize ambient
#[wasm_bindgen]
pub fn init(settings: JsValue) -> Result<(), JsValue> {
    let settings: Settings = serde_wasm_bindgen::from_value(settings)?;

    if settings.enable_logging {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_ansi(true) // Only partially supported, but works on Chrome
            .without_time()
            .with_writer(MakeConsoleWriter); // write events to the console

        registry().with(LevelFilter::DEBUG).with(fmt_layer).init();
    }

    if settings.enable_panic_hook {
        ambient_sys::set_panic_hook();
    }

    tracing::info!("Hello, Wasm!");

    ambient_ecs::init_components();
    ambient_core::init_all_components();
    // ambient_water::init_components();
    ambient_network::init_all_components();
    ambient_world_audio::init_components();
    ambient_wasm::shared::init_all_components();
    ambient_primitives::init_components();
    ambient_package_semantic_native::init_components();

    Ok(())
}

#[wasm_bindgen]
/// Starts execution of the ambient client and connects to the specified URL
///
/// TODO: The `MainApp` setup will move to an Ambient package and this will only load the runtime
pub async fn start(target: Option<web_sys::HtmlElement>, server_url: String) {
    if let Err(err) = run(target, server_url).await {
        tracing::error!("{err:?}")
    }
}

#[wasm_bindgen]
pub fn stop() {
    if let Some(ctl_tx) = APP_CONTROL.get() {
        let _ = ctl_tx.send(WindowCtl::ExitProcess(ExitStatus::SUCCESS));
    } else {
        tracing::warn!("App not initialized");
    }
}

async fn run(target: Option<web_sys::HtmlElement>, server_url: String) -> anyhow::Result<()> {
    let (ctl_tx, ctl_rx) = flume::unbounded();

    APP_CONTROL
        .set(ctl_tx.clone())
        .map_err(|_| anyhow::Error::msg("App already initialized"))?;

    use ambient_sys::timer::TimerWheel;
    ambient_sys::task::spawn(TimerWheel::new().start());

    use anyhow::Context;
    let mut app = App::builder()
        .ui_renderer(true)
        .parent_element(target)
        .window_ctl(ctl_tx, ctl_rx)
        .build()
        .await
        .context("Failed to build app")?;

    tracing::info!("Finished building app");

    let world = &mut app.world;

    Group(vec![
        UICamera.el().with(active_camera(), 0.),
        ambient_client_shared::player::PlayerRawInputHandler.el(),
        WindowSized::el([MainApp::el(server_url).with(padding(), Borders::even(10.).into())]),
    ])
    .el()
    .spawn_interactive(world);

    // Spawn the event loop
    app.spawn();

    Ok(())
}
