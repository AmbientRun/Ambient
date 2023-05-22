use ambient_app::App;
use ambient_audio::*;
use ambient_cameras::UICamera;
use ambient_core::camera::active_camera;
use ambient_ecs::{Entity, SystemGroup};
use ambient_network::{server::RpcArgs, web::GameClientView};
use ambient_renderer::color;
use ambient_rpc::RpcRegistry;
use ambient_std::{cb, friendly_id};
use ambient_sys::time::Instant;
use ambient_ui_native::{
    element::{element_component, Element, ElementComponentExt, Group, Hooks},
    font_size, padding, space_between_items, Borders, Button, Dock, FlowColumn, FocusRoot,
    Separator, StylesExt, Text, TextEditor, UIExt,
};
use app::MainApp;
use game_view::GameView;
use glam::vec4;
use js_sys::{Float32Array, Function};
use std::cell::RefCell;
use std::rc::Rc;
use std::{collections::HashMap, time::Duration};
use tracing_subscriber::{filter::LevelFilter, fmt::time::UtcTime, prelude::*, registry};
use tracing_web::MakeConsoleWriter;
use url::Url;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

mod app;
mod game_view;

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

#[wasm_bindgen]
/// Initialize ambient
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
}

#[wasm_bindgen]
/// Starts execution of the ambient client
pub async fn start(target: Option<web_sys::HtmlElement>) {
    if let Err(err) = run(target).await {
        tracing::error!("{err:?}")
    }
}

async fn run(target: Option<web_sys::HtmlElement>) -> anyhow::Result<()> {
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

    init(&mut app).await;

    // Spawn the event loop
    app.spawn();

    Ok(())
}

async fn init(app: &mut App) {
    let world = &mut app.world;

    Group(vec![
        UICamera.el().with(active_camera(), 0.),
        FocusRoot(vec![
            MainApp::el().with(padding(), Borders::even(10.).into())
        ])
        .el(),
    ])
    .el()
    .spawn_interactive(world);
}
