use ambient_app::App;
use ambient_core::name;
use ambient_ecs::{EntityData, World};
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

    let mut world = World::new("main");

    ambient_core::init_all_components();
    let id = EntityData::new().set(name(), "wasm-entity".into()).spawn(&mut world);

    tracing::info!("Spawned {id}");

    if let Err(err) = run().await {
        tracing::error!("{err:?}")
    }
}

#[cfg(target_os = "unknown")]
async fn run() -> anyhow::Result<()> {
    use anyhow::Context;
    App::builder().build().await.context("Failed to build app")?.spawn();

    Ok(())
}

#[cfg(not(target_os = "unknown"))]
async fn run() -> anyhow::Result<()> {
    unimplemented!("This only builds on the web")
}
