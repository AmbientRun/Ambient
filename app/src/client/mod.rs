use std::{
    collections::HashMap, path::PathBuf, process::exit, sync::Arc, time::Duration,
};

use ambient_app::{fps_stats, window_title, AppBuilder};
use ambient_cameras::UICamera;
use ambient_core::{
    runtime,
    window::{
        cursor_position, window_ctl, window_logical_size, window_physical_size,
        window_scale_factor, WindowCtl,
    },
};
use ambient_debugger::Debugger;
use ambient_ecs::{Entity, EntityId, SystemGroup};
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_network::{
    client::{client_network_stats, GameClient, GameClientRenderTarget, GameClientWorld},
    hooks::use_remote_resource,
    native::client::{GameClientView, ResolvedAddr},
};
use ambient_std::{asset_cache::AssetCache, cb, friendly_id};
use ambient_sys::time::Instant;
use ambient_ui_native::{
    Button, Dock, FlowColumn, FocusRoot, MeasureSize, ScrollArea, ScrollAreaSizing, StylesExt,
    Text, UIExt, WindowSized, STREET,
};
use glam::{uvec2, vec4, Vec2};

use crate::{
    cli::{GoldenImageCommand, RunCli},
    shared,
};
use ambient_ecs_editor::{ECSEditor, InspectableAsyncWorld};
use ambient_layout::{docking, padding, Borders};

pub mod player;
mod wasm;

/// Construct an app and enter the main client view
pub async fn run(
    assets: AssetCache,
    server_addr: ResolvedAddr,
    run: &RunCli,
    golden_image_output_dir: Option<PathBuf>,
) {
    let user_id = run
        .user_id
        .clone()
        .unwrap_or_else(|| format!("user_{}", friendly_id()));
    let headless = if run.headless {
        Some(uvec2(600, 600))
    } else {
        None
    };

    let is_debug = std::env::var("AMBIENT_DEBUGGER").is_ok() || run.debugger;

    let cert = if let Some(ca) = &run.ca {
        match std::fs::read(ca) {
            Ok(v) => Some(v),
            Err(err) => {
                tracing::error!("Failed to load certificate from file: {}", err);
                None
            }
        }
    } else {
        #[cfg(not(feature = "no_bundled_certs"))]
        {
            Some(super::CERT.to_vec())
        }
        #[cfg(feature = "no_bundled_certs")]
        {
            None
        }
    };

    AppBuilder::new()
        .ui_renderer(true)
        .with_asset_cache(assets)
        .headless(headless)
        .update_title_with_fps_stats(false)
        .run(move |app, _runtime| {
            *app.world.resource_mut(window_title()) = "Ambient".to_string();
            MainApp {
                server_addr,
                user_id,
                show_debug: is_debug,
                golden_image_cmd: run.golden_image,
                golden_image_output_dir,
                cert,
            }
            .el()
            .spawn_interactive(&mut app.world);
        })
        .await;
}

#[element_component]
fn TitleUpdater(hooks: &mut Hooks) -> Element {
    let (net, _) = use_remote_resource(hooks, client_network_stats()).expect("No game client");

    let world = &hooks.world;
    let title = world.resource(window_title());
    let fps = world
        .get_cloned(hooks.world.resource_entity(), fps_stats())
        .ok()
        .filter(|f| !f.fps().is_nan());

    let title = match (fps, net) {
        (None, None) => title.clone(),
        (Some(fps), None) => format!("{} [{}]", title, fps.dump_both()),
        (None, Some(net)) => format!("{} [{}]", title, net),
        (Some(fps), Some(net)) => format!("{} [{}, {}]", title, fps.dump_both(), net),
    };
    world
        .resource(window_ctl())
        .send(WindowCtl::SetTitle(title))
        .ok();

    Element::new()
}

#[element_component]
fn MainApp(
    hooks: &mut Hooks,
    server_addr: ResolvedAddr,
    golden_image_output_dir: Option<PathBuf>,
    user_id: String,
    show_debug: bool,
    golden_image_cmd: Option<GoldenImageCommand>,
    cert: Option<Vec<u8>>,
) -> Element {
    let (loaded, set_loaded) = hooks.use_state(false);

    FocusRoot::el([
        UICamera.el(),
        player::PlayerRawInputHandler.el(),
        WindowSized::el([GameClientView {
            server_addr,
            user_id,
            on_loaded: cb(move |client| {
                let mut game_state = client.game_state.lock();
                let world = &mut game_state.world;

                wasm::initialize(world).unwrap();

                UICamera.el().spawn_static(world);
                set_loaded(true);

                Ok(Box::new(|| {
                    log::info!("Disconnecting client");
                }))
            }),
            error_view: cb(move |error| {
                Dock(vec![Text::el("Error").header_style(), Text::el(error)]).el()
            }),
            systems_and_resources: cb(|| {
                let mut resources = Entity::new();

                let bistream_handlers = HashMap::new();
                resources.set(
                    ambient_network::client::bi_stream_handlers(),
                    bistream_handlers,
                );

                let unistream_handlers = HashMap::new();
                resources.set(
                    ambient_network::client::uni_stream_handlers(),
                    unistream_handlers,
                );

                let dgram_handlers = HashMap::new();
                resources.set(ambient_network::client::datagram_handlers(), dgram_handlers);

                (systems(), resources)
            }),
            cert,
            create_rpc_registry: cb(shared::create_server_rpc_registry),
            inner: Dock::el(vec![
                TitleUpdater.el(),
                if let Some(golden_image_cmd) = golden_image_cmd.filter(|_| loaded) {
                    GoldenImageTest::el(golden_image_output_dir, golden_image_cmd)
                } else {
                    Element::new()
                },
                // Text::el("Insert game here"),
                GameView { show_debug }.el(),
            ]),
        }
        .el()]),
    ])
}

#[element_component]
fn GoldenImageTest(
    hooks: &mut Hooks,
    golden_image_output_dir: Option<PathBuf>,
    golden_image_cmd: GoldenImageCommand,
) -> Element {
    let (render_target, _) = hooks.consume_context::<GameClientRenderTarget>().unwrap();
    let render_target_ref = hooks.use_ref_with(|_| render_target.clone());
    *render_target_ref.lock() = render_target.clone();
    let screenshot_path = golden_image_output_dir
        .unwrap_or(PathBuf::new())
        .join("screenshot.png");
    let (old_screenshot, _) = hooks.use_state_with(|_| {
        tracing::info!("Loading screenshot from {:?}", screenshot_path);
        Some(Arc::new(image::open(&screenshot_path).ok()?))
    });
    if matches!(
        golden_image_cmd,
        GoldenImageCommand::GoldenImageCheck { .. }
    ) && old_screenshot.is_none()
    {
        panic!(
            "Failed golden image check: existing screenshot must exist at '{}'. \
            Consider running the test with --golden-image update --wait-seconds 5",
            screenshot_path.display()
        );
    }

    let rt = hooks.world.resource(runtime()).clone();
    match golden_image_cmd {
        GoldenImageCommand::GoldenImageUpdate { wait_seconds } => {
            hooks.use_spawn(move |world| {
                world.resource(runtime()).spawn(async move {
                    tokio::time::sleep(Duration::from_secs_f32(wait_seconds)).await;
                    let render_target = render_target_ref.lock().clone();
                    tracing::info!("Saving screenshot to {:?}", screenshot_path);
                    let new = render_target
                        .0
                        .color_buffer
                        .reader()
                        .read_image()
                        .await
                        .unwrap()
                        .into_rgba8();
                    tracing::info!("Screenshot saved");
                    new.save(screenshot_path).unwrap();
                    exit(0);
                });

                |_| {}
            });
        }

        GoldenImageCommand::GoldenImageCheck { timeout_seconds } => {
            if old_screenshot.is_none() {
                panic!("Existing screenshot must exist")
            }

            let start_time = Instant::now();
            hooks.use_interval_deps(
                Duration::from_secs_f32(0.25),
                false,
                render_target.0.color_buffer.id,
                {
                    move |_| {
                        if let Some(old) = old_screenshot.clone() {
                            let render_target = render_target.clone();
                            rt.spawn(async move {
                                if start_time.elapsed().as_secs_f32() > timeout_seconds {
                                    exit(1);
                                }

                                let new = render_target
                                    .0
                                    .color_buffer
                                    .reader()
                                    .read_image()
                                    .await
                                    .unwrap()
                                    .into_rgba8();

                                let hasher = image_hasher::HasherConfig::new().to_hasher();

                                let hash1 = hasher.hash_image(&new);
                                let hash2 = hasher.hash_image(&*old);
                                let dist = hash1.dist(&hash2);
                                if dist <= 2 {
                                    tracing::info!("Screenshots are identical, exiting");
                                    exit(0);
                                } else {
                                    tracing::warn!("Screenshot differ, distance={dist}");
                                }
                            });
                        }
                    }
                },
            );
        }
    }

    Element::new()
}

#[element_component]
fn GameView(hooks: &mut Hooks, show_debug: bool) -> Element {
    let (state, _) = hooks.consume_context::<GameClient>().unwrap();
    let (render_target, _) = hooks.consume_context::<GameClientRenderTarget>().unwrap();
    let (show_ecs, set_show_ecs) = hooks.use_state(true);
    let (ecs_size, set_ecs_size) = hooks.use_state(Vec2::ZERO);

    hooks.use_frame({
        let state = state.clone();
        let render_target = render_target.clone();
        move |world| {
            let mut state = state.game_state.lock();
            let scale_factor = *world.resource(window_scale_factor());
            let mut mouse_pos = *world.resource(cursor_position());
            mouse_pos.x -= ecs_size.x;
            state
                .world
                .set_if_changed(EntityId::resources(), cursor_position(), mouse_pos)
                .unwrap();
            let size = uvec2(
                render_target.0.color_buffer.size.width,
                render_target.0.color_buffer.size.height,
            );
            state
                .world
                .set_if_changed(
                    EntityId::resources(),
                    window_logical_size(),
                    (size.as_vec2() / scale_factor as f32).as_uvec2(),
                )
                .unwrap();
            state
                .world
                .set_if_changed(EntityId::resources(), window_physical_size(), size)
                .unwrap();
            state
                .world
                .set_if_changed(EntityId::resources(), window_scale_factor(), scale_factor)
                .unwrap();
        }
    });

    Dock::el([
        if show_debug {
            MeasureSize::el(
                FlowColumn::el([
                    Button::new(if show_ecs { "\u{f137}" } else { "\u{f138}" }, move |_| {
                        set_show_ecs(!show_ecs)
                    })
                    .style(ambient_ui_native::ButtonStyle::Flat)
                    .toggled(show_ecs)
                    .el(),
                    if show_ecs {
                        ScrollArea::el(
                            ScrollAreaSizing::FitChildrenWidth,
                            ECSEditor {
                                world: Arc::new(InspectableAsyncWorld(cb({
                                    let state = state.clone();
                                    move |res| {
                                        let state = state.game_state.lock();
                                        res(&state.world)
                                    }
                                }))),
                            }
                            .el()
                            .memoize_subtree(state.uid),
                        )
                    } else {
                        Element::new()
                    },
                ])
                .with(docking(), ambient_layout::Docking::Left)
                .with_background(vec4(0., 0., 0., 1.))
                .with(padding(), Borders::even(STREET).into()),
                set_ecs_size,
            )
        } else {
            Element::new()
        },
        if show_debug {
            Debugger {
                get_state: cb(move |cb| {
                    let mut game_state = state.game_state.lock();
                    let game_state = &mut *game_state;
                    cb(
                        &mut game_state.renderer,
                        &render_target.0,
                        &mut game_state.world,
                    );
                }),
            }
            .el()
            .with(docking(), ambient_layout::Docking::Bottom)
            .with(padding(), Borders::even(STREET).into())
        } else {
            Element::new()
        },
        if show_debug {
            Dock::el([GameClientWorld.el()])
                .with_background(vec4(0.2, 0.2, 0.2, 1.))
                .with(
                    padding(),
                    Borders {
                        left: 1.,
                        top: 0.,
                        right: 0.,
                        bottom: 1.,
                    }
                    .into(),
                )
        } else {
            GameClientWorld.el()
        },
    ])
}

fn systems() -> SystemGroup {
    SystemGroup::new(
        "client",
        vec![
            Box::new(ambient_prefab::systems()),
            Box::new(ambient_decals::client_systems()),
            Box::new(ambient_primitives::systems()),
            Box::new(ambient_sky::systems()),
            Box::new(ambient_water::systems()),
            Box::new(ambient_physics::client_systems()),
            Box::new(wasm::systems()),
            Box::new(player::systems_final()),
        ],
    )
}
