use std::{collections::HashMap, time::Duration};

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
use game_view::GameView;
use glam::vec4;
use js_sys::{Float32Array, Function};
use std::cell::RefCell;
use std::rc::Rc;
use tracing_subscriber::{filter::LevelFilter, fmt::time::UtcTime, prelude::*, registry};
use tracing_web::MakeConsoleWriter;
use url::Url;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

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

#[element_component]
pub fn View(hooks: &mut Hooks) -> Element {
    let now = Instant::now();
    let (elapsed, set_elapsed) = hooks.use_state(Duration::ZERO);
    hooks.use_interval(0.2, move || set_elapsed(now.elapsed()));

    FlowColumn(vec![
        Text::el(format!(
            "Hello from the browser! {:.2}",
            elapsed.as_secs_f32()
        ))
        .header_style(),
        // GameClientView {
        //     url: Url::parse("https://127.0.0.1:9000").unwrap(),
        //     user_id: friendly_id(),
        //     systems_and_resources: cb(|| {
        //         let mut resources = Entity::new();

        //         let bistream_handlers = HashMap::new();
        //         resources.set(
        //             ambient_network::client::bi_stream_handlers(),
        //             bistream_handlers,
        //         );

        //         let unistream_handlers = HashMap::new();
        //         resources.set(
        //             ambient_network::client::uni_stream_handlers(),
        //             unistream_handlers,
        //         );

        //         let dgram_handlers = HashMap::new();
        //         resources.set(ambient_network::client::datagram_handlers(), dgram_handlers);

        //         (systems(), resources)
        //     }),
        //     on_loaded: cb(move |client| {
        //         let mut game_state = client.game_state.lock();
        //         let world = &mut game_state.world;

        //         // wasm::initialize(world).unwrap();

        //         UICamera.el().spawn_static(world);

        //         Ok(Box::new(|| {
        //             tracing::info!("Disconnecting client");
        //         }))
        //     }),
        //     create_rpc_registry: cb(create_server_rpc_registry),
        //     inner: Dock::el(vec![GameView { show_debug: false }.el()]),
        // }
        // .el(),
        Button::new("Play audio", move |_| {
            let mut sine = SineWave::new(440.0);
            // let mut phase = 0.0;
            let window = web_sys::window().expect("no global `window` exists");
            let this = JsValue::null();
            let start = window
                .get("audioStart")
                .unwrap()
                .dyn_into::<Function>()
                .unwrap();
            start.call0(&this).unwrap();

            let sab = window
                .get("dataSAB")
                .unwrap()
                .dyn_into::<js_sys::SharedArrayBuffer>()
                .unwrap();
            let buf = Float32Array::new(&sab);
            let write_ptr = JsValue::from(
                window
                    .get("writePtr")
                    .unwrap()
                    .dyn_into::<js_sys::Uint32Array>()
                    .unwrap(),
            ); //.dyn_into::<js_sys::Uint32Array>().unwrap();
            let read_ptr = JsValue::from(
                window
                    .get("readPtr")
                    .unwrap()
                    .dyn_into::<js_sys::Uint32Array>()
                    .unwrap(),
            ); //.dyn_into::<js_sys::Uint32Array>().unwrap();

            let f = Rc::new(RefCell::new(None));
            let g = f.clone();
            *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
                loop {
                    let wr = js_sys::Atomics::load(&write_ptr, 0).unwrap();
                    let rd = js_sys::Atomics::load(&read_ptr, 0).unwrap();
                    let available_read = (wr + 2048 - rd) % 2048;
                    let available_write = 2048 - available_read;
                    if available_write <= 128 {
                        break;
                    }
                    for _ in 0..128 {
                        let wr = js_sys::Atomics::load(&write_ptr, 0).unwrap();
                        // let val = (phase * 2.0 * std::f32::consts::PI).sin();
                        let val = sine.next_sample().unwrap()[0];
                        buf.set_index(wr as u32, val);
                        js_sys::Atomics::store(&write_ptr, 0, (wr + 1) % (2048)).unwrap();
                        // phase += 440.0 / 44100.0;
                        // if phase > 1.0 {
                        //     phase -= 1.0;
                        // }
                    }
                }
                request_animation_frame(f.borrow().as_ref().unwrap());
            }) as Box<dyn FnMut()>));

            request_animation_frame(g.borrow().as_ref().unwrap());
        })
        .el()
        .with_background(vec4(1.0, 0.5, 0.5, 1.0)),
        Button::new("Stop audio", move |_| {
            let window = web_sys::window().expect("no global `window` exists");
            let this = JsValue::null();
            let stop = window
                .get("audioStop")
                .unwrap()
                .dyn_into::<Function>()
                .unwrap();
            stop.call0(&this).unwrap();
        })
        .el()
        .with_background(vec4(0.0, 0.5, 0.9, 1.0)),
    ])
    .el()
    .with(space_between_items(), 10.)
}

async fn init(app: &mut App) {
    let world = &mut app.world;

    Group(vec![
        UICamera.el().with(active_camera(), 0.),
        FocusRoot(vec![View::el().with(padding(), Borders::even(10.).into())]).el(),
    ])
    .el()
    .spawn_interactive(world);
}

/// Declares the systems to run in the network client world
fn systems() -> SystemGroup {
    SystemGroup::new(
        "client",
        vec![
            // Box::new(ambient_prefab::systems()),
            // Box::new(ambient_decals::client_systems()),
            // Box::new(ambient_primitives::systems()),
            // Box::new(ambient_sky::systems()),
            // Box::new(ambient_water::systems()),
            // Box::new(ambient_physics::client_systems()),
            // Box::new(wasm::systems()),
            // Box::new(player::systems_final()),
        ],
    )
}

pub fn create_server_rpc_registry() -> RpcRegistry<RpcArgs> {
    let mut reg = RpcRegistry::new();
    ambient_network::rpc::register_server_rpcs(&mut reg);
    // ambient_debugger::register_server_rpcs(&mut reg);
    reg
}
