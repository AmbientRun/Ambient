use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};

use ambient_app::{fps_stats, window_title, AppBuilder};
use ambient_audio::AudioMixer;
use ambient_cameras::UICamera;
use ambient_client_shared::game_view::GameView;
use ambient_core::{
    gpu, runtime,
    window::{window_ctl, ExitStatus, WindowCtl},
};
use ambient_ecs::{Entity, SystemGroup};
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_native_std::{asset_cache::AssetCache, cb, friendly_id};
use ambient_network::{
    client::{client_network_stats, GameClientRenderTarget},
    hooks::use_remote_resource,
    native::client::{ClientView, ResolvedAddr},
};
use ambient_sys::time::Instant;
use ambient_ui_native::{Dock, FocusRoot, WindowSized};
use glam::uvec2;

use crate::{
    cli::{GoldenImageCommand, RunCli},
    shared::{self, certs::CERT},
};

mod wasm;

/// Construct an app and enter the main client view
pub async fn run(
    assets: AssetCache,
    server_addr: ResolvedAddr,
    run: &RunCli,
    golden_image_output_dir: Option<PathBuf>,
    mixer: Option<AudioMixer>,
) -> ExitStatus {
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
            Some(CERT.to_vec())
        }
        #[cfg(feature = "no_bundled_certs")]
        {
            None
        }
    };

    log::info!(
        "server_addr, golden_image_output_dir, {:?} {:?}",
        server_addr,
        golden_image_output_dir,
    );

    AppBuilder::new()
        // .ui_renderer(true)
        // .with_asset_cache(assets)
        // .headless(headless)
        // .update_title_with_fps_stats(false)
        .run(move |app, _runtime| {
            // *app.world.resource_mut(window_title()) = "Ambient".to_string();
            MainApp {
                server_addr,
                user_id,
                show_debug: is_debug,
                golden_image_cmd: run.golden_image,
                golden_image_output_dir,
                cert,
                mixer,
            }
            .el()
            .spawn_interactive(&mut app.world);
        })
        .await
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
    mixer: Option<AudioMixer>,
) -> Element {
    Element::new()
    // let (loaded, set_loaded) = hooks.use_state(false);

    // FocusRoot::el([
    //     UICamera.el(),
    //     ambient_client_shared::player::PlayerRawInputHandler.el(),
    //     WindowSized::el([ClientView {
    //         server_addr,
    //         user_id,
    //         // NOTE: client.game_state is **locked** and accesible through game_state.
    //         //
    //         // This is to prevent another thread from updating using the client after connection but
    //         // just before `on_loaded`. This is a very small window of time, but does occasionally
    //         // happen, especially when joining a server which is already running and finished
    //         // loading.
    //         on_loaded: cb(move |_, game_state| {
    //             let world = &mut game_state.world;

    //             wasm::initialize(world, mixer.clone()).unwrap();

    //             UICamera.el().spawn_static(world);
    //             set_loaded(true);

    //             Ok(Box::new(|| {
    //                 log::info!("Disconnecting client");
    //             }))
    //         }),
    //         systems_and_resources: cb(|| {
    //             let mut resources = Entity::new();

    //             let bistream_handlers = HashMap::new();
    //             resources.set(
    //                 ambient_network::client::bi_stream_handlers(),
    //                 bistream_handlers,
    //             );

    //             let unistream_handlers = HashMap::new();
    //             resources.set(
    //                 ambient_network::client::uni_stream_handlers(),
    //                 unistream_handlers,
    //             );

    //             let dgram_handlers = HashMap::new();
    //             resources.set(ambient_network::client::datagram_handlers(), dgram_handlers);

    //             (systems(), resources)
    //         }),
    //         cert,
    //         create_rpc_registry: cb(shared::create_server_rpc_registry),
    //         inner: Dock::el(vec![
    //             TitleUpdater.el(),
    //             if let Some(golden_image_cmd) = golden_image_cmd.filter(|_| loaded) {
    //                 GoldenImageTest::el(golden_image_output_dir, golden_image_cmd)
    //             } else {
    //                 Element::new()
    //             },
    //             GameView { show_debug }.el(),
    //         ]),
    //     }
    //     .el()]),
    // ])
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
    let golden_image_output_dir = golden_image_output_dir.unwrap_or(PathBuf::new());
    let screenshot_path = golden_image_output_dir.join("screenshot.png");
    let fail_screenshot_path = golden_image_output_dir.join("fail_screenshot.png");
    let (old_screenshot, _) = hooks.use_state_with(|_| {
        tracing::info!("Loading screenshot from {:?}", screenshot_path);
        Some(Arc::new(image::open(&screenshot_path).ok()?))
    });
    if matches!(golden_image_cmd, GoldenImageCommand::Check { .. }) && old_screenshot.is_none() {
        panic!(
            "Failed golden image check: existing screenshot must exist at '{}'. \
            Consider running the test with --golden-image update --wait-seconds 5",
            screenshot_path.display()
        );
    }

    match golden_image_cmd {
        GoldenImageCommand::Update { wait_seconds } => {
            hooks.use_spawn(move |world| {
                let window_ctl = world.resource(window_ctl()).clone();
                let gpu = world.resource(gpu()).clone();
                world.resource(runtime()).spawn(async move {
                    // Wait until image is sufficiently converged.
                    tokio::time::sleep(Duration::from_secs_f32(wait_seconds)).await;

                    // Capture current frame.
                    let render_target = render_target_ref.lock().clone();
                    let new = render_target
                        .0
                        .color_buffer
                        .reader(&gpu)
                        .read_image(&gpu)
                        .await
                        .unwrap()
                        .into_rgba8();

                    // Save to disk.
                    new.save(&screenshot_path).unwrap();
                    tracing::info!(
                        "Saved screenshot to {}, exiting with 0",
                        screenshot_path.display()
                    );

                    // Graceful exit.
                    window_ctl
                        .send(WindowCtl::ExitProcess(ExitStatus::SUCCESS))
                        .unwrap();
                });

                |_| {}
            });
        }

        GoldenImageCommand::Check { timeout_seconds } => {
            let Some(old_screenshot) = old_screenshot else {
                panic!("Existing screenshot must exist");
            };

            // Note: this is basically hooks.use_interval_deps() except its
            // internals are unwrapped in order to access the `world`, which we
            // need for window_ctl().
            hooks.use_effect(render_target.0.color_buffer.id, move |world, _| {
                let window_ctl = world.resource(window_ctl()).clone();
                let gpu = world.resource(gpu()).clone();
                let start_time = Instant::now();
                let task = world.resource(runtime()).spawn(async move {
                    let mut interval = ambient_sys::time::interval(Duration::from_secs_f32(0.25));
                    interval.tick().await;
                    loop {
                        interval.tick().await;

                        // Capture current frame.
                        let new = render_target
                            .0
                            .color_buffer
                            .reader(&gpu)
                            .read_image(&gpu)
                            .await
                            .unwrap()
                            .into_rgba8();

                        // Handle timeout.
                        if start_time.elapsed().as_secs_f32() > timeout_seconds {
                            tracing::error!(
                                "Golden image check timed out after {timeout_seconds} seconds!"
                            );

                            // Save failed image to disk for later analysis.
                            new.save(&fail_screenshot_path).unwrap();
                            tracing::error!(
                                "Wrote last frame to {}, exiting with 1",
                                fail_screenshot_path.display()
                            );

                            // Graceful exit.
                            window_ctl
                                .send(WindowCtl::ExitProcess(ExitStatus::FAILURE))
                                .unwrap();
                            break;
                        }

                        // Perceptual image difference.
                        // Todo: replace with NVIDIA FLIP.
                        let hasher = image_hasher::HasherConfig::new().to_hasher();
                        let hash1 = hasher.hash_image(&new);
                        let hash2 = hasher.hash_image(&*old_screenshot);
                        let dist = hash1.dist(&hash2);
                        if dist <= 3 {
                            tracing::info!("Screenshots are identical, exiting with 0");

                            // Graceful exit.
                            window_ctl
                                .send(WindowCtl::ExitProcess(ExitStatus::SUCCESS))
                                .unwrap();
                            break;
                        } else {
                            tracing::warn!("Screenshot differ, distance={dist}");
                        }
                    }
                });

                move |_| {
                    task.abort();
                }
            });
        }
    }

    Element::new()
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
            Box::new(ambient_gizmos::client_systems()),
            Box::new(wasm::systems()),
            Box::new(ambient_client_shared::player::systems_final()),
        ],
    )
}
