use std::sync::OnceLock;

use ambient_app::App;
use ambient_cameras::UICamera;
use ambient_core::{
    camera::active_camera,
    window::{ExitStatus, WindowCtl},
};
use ambient_ui_native::{
    element::{ElementComponentExt, Group},
    WindowSized,
};
use app::MainApp;
use tracing_subscriber::{
    filter::{LevelFilter, Targets},
    prelude::*,
    registry,
};
use tracing_web::MakeConsoleWriter;
use wasm_bindgen::prelude::*;

mod app;
mod wasm;

static APP_CONTROL: OnceLock<flume::Sender<WindowCtl>> = OnceLock::new();

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
/// This is inteded to be used by a loosely typed settings object from javascript allow for
/// backwards and forwards compatibility when new fields are added to the settings object in a
/// "best fit" approach, similar to the `settings` in the LanguageServerProtocol spec.
pub struct Settings {
    enable_logging: bool,
    enable_panic_hook: bool,
    allow_version_mismatch: bool,
    log_filter: Option<String>,
    debugger: bool,
}

#[wasm_bindgen]
/// Starts execution of the ambient client and connects to the specified URL
///
/// TODO: The `MainApp` setup will move to an Ambient package and this will only load the runtime
pub async fn start(target: Option<web_sys::HtmlElement>, server_url: String, settings: JsValue) {
    let settings: Settings =
        serde_wasm_bindgen::from_value(settings).expect("settings can be parsed from object");

    init(&settings).expect("Failed to initialize ambient");

    if let Err(err) = run(target, server_url, settings).await {
        tracing::error!("{err:?}")
    }
}

/// Initialize ambient
fn init(settings: &Settings) -> Result<(), JsValue> {
    if settings.enable_logging {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_ansi(true) // Only partially supported, but works on Chrome
            .without_time()
            .with_writer(MakeConsoleWriter); // write events to the console

        let filter = settings
            .log_filter
            .as_ref()
            .map(|v| {
                v.parse::<Targets>()
                    .map_err(|_| JsValue::from("Failed to parse filtering directive"))
            })
            .transpose()?
            .unwrap_or_else(|| Targets::new().with_default(LevelFilter::INFO));

        registry().with(filter).with(fmt_layer).init();
    }

    if settings.enable_panic_hook {
        ambient_sys::set_panic_hook();
    }

    ambient_ecs::init_components();
    ambient_core::init_all_components();
    ambient_water::init_components();
    ambient_sky::init_components();
    ambient_network::init_all_components();
    ambient_world_audio::init_components();
    ambient_wasm::shared::init_all_components();
    ambient_decals::init_components();
    ambient_primitives::init_components();
    ambient_package_semantic_native::init_components();

    Ok(())
}

#[wasm_bindgen]
pub fn stop() {
    if let Some(ctl_tx) = APP_CONTROL.get() {
        let _ = ctl_tx.send(WindowCtl::ExitProcess(ExitStatus::SUCCESS));
    } else {
        tracing::warn!("App not initialized");
    }
}

async fn run(
    target: Option<web_sys::HtmlElement>,
    server_url: String,
    settings: Settings,
) -> anyhow::Result<()> {
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
        WindowSized::el([MainApp::el(server_url, settings)]),
    ])
    .el()
    .spawn_interactive(world);

    // Spawn the event loop
    app.spawn();

    Ok(())
}
