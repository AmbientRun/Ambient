use ambient_app::App;
use ambient_cameras::UICamera;
use ambient_core::camera::active_camera;
use ambient_ui_native::{
    element::{ElementComponentExt, Group},
    padding, Borders,  WindowSized,
};
use app::MainApp;
use tracing_subscriber::{filter::LevelFilter, fmt::time::UtcTime, prelude::*, registry};
use tracing_web::MakeConsoleWriter;
use wasm_bindgen::prelude::*;

mod app;
mod wasm;

/// Initialize ambient
#[wasm_bindgen]
pub fn init_ambient(logging: bool, panic: bool) {
    if logging {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_ansi(false) // Only partially supported across browsers
            .with_timer(UtcTime::rfc_3339()) // std::time is not available in browsers
            .with_writer(MakeConsoleWriter); // write events to the console

        registry().with(LevelFilter::DEBUG).with(fmt_layer).init();
    }

    if panic {
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

async fn run(target: Option<web_sys::HtmlElement>, server_url: String) -> anyhow::Result<()> {
    use ambient_sys::timer::TimerWheel;
    ambient_sys::task::spawn(TimerWheel::new().start());

    use anyhow::Context;
    let mut app = App::builder()
        .ui_renderer(true)
        .parent_element(target)
        .build()
        .await
        .context("Failed to build app")?;

    tracing::info!("Finished building app");

    init(&mut app, server_url).await;

    // Spawn the event loop
    app.spawn();

    Ok(())
}

async fn init(app: &mut App, server_url: String) {
    let world = &mut app.world;

    Group(vec![
        UICamera.el().with(active_camera(), 0.),
        ambient_client_shared::player::PlayerRawInputHandler.el(),
        WindowSized::el([
            MainApp::el(server_url).with(padding(), Borders::even(10.).into())
        ])
        .el(),
    ])
    .el()
    .spawn_interactive(world);
}
