use ambient_app::App;
use ambient_cameras::UICamera;
use ambient_core::camera::active_camera;
use ambient_renderer::color;
use ambient_ui::{
    element::{ElementComponentExt, Group},
    font_size, padding, space_between_items, Borders, FlowColumn, FlowRow, FocusRoot, Separator, StylesExt, Text,
};
use glam::vec4;
use tracing_subscriber::{filter::LevelFilter, fmt::time::UtcTime, prelude::*, registry};
use tracing_web::MakeConsoleWriter;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(start)]
async fn start() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false) // Only partially supported across browsers
        .with_timer(UtcTime::rfc_3339()) // std::time is not available in browsers
        .with_writer(MakeConsoleWriter); // write events to the console

    registry().with(LevelFilter::DEBUG).with(fmt_layer).init();

    ambient_sys::set_panic_hook();

    tracing::info!("Hello, Wasm!");

    ambient_core::init_all_components();

    if let Err(err) = run().await {
        tracing::error!("{err:?}")
    }
}

#[cfg(target_os = "unknown")]
async fn run() -> anyhow::Result<()> {
    use ambient_sys::timer::TimerWheel;
    ambient_sys::task::spawn(TimerWheel::new().start());

    use anyhow::Context;
    let mut app = App::builder().ui_renderer(true).build().await.context("Failed to build app")?;
    tracing::info!("Finished building app");

    init(&mut app);

    /// Spawn the event loop
    app.spawn();
    Ok(())
}

async fn init(app: &mut App) {
    let world = &mut app.world;
    // FocusRoot(vec![FlowRow(vec![Text::el("Hello, Wasm!")]).el()]).el().spawn_interactive(world);

    Group(vec![
        UICamera.el().set(active_camera(), 0.),
        FlowColumn(vec![
            Text::el("Header").header_style(),
            Text::el("Section").section_style(),
            Text::el("Default text \u{f1e2} \u{fb8f}"),
            Text::el("Small").small_style(),
            Separator { vertical: false }.el(),
            Text::el("Custom size").set(font_size(), 40.),
            Text::el("Custom color").set(color(), vec4(1., 0., 0., 1.)),
            Text::el("Multi\n\nLine"),
        ])
        .el()
        .set(padding(), Borders::even(10.))
        .set(space_between_items(), 10.),
    ])
    .el()
    .spawn_interactive(world);
}

#[cfg(not(target_os = "unknown"))]
async fn run() -> anyhow::Result<()> {
    unimplemented!("This only builds on the web")
}
