use std::{fmt::Debug, sync::Arc, time::Duration};

use ambient_core::{
    asset_cache,
    bounding::world_bounding_sphere,
    camera::shadow_cameras_from_world,
    hierarchy::{dump_world_hierarchy, dump_world_hierarchy_to_user},
    main_scene, performance_samples,
    player::local_user_id,
    runtime,
};
use ambient_ecs::{query, World};
use ambient_element::{
    consume_context, element_component, use_frame, use_state, use_state_with, Element,
    ElementComponentExt, Hooks,
};
use ambient_gizmos::{gizmos, GizmoPrimitive};
use ambient_native_std::{asset_cache::AssetCache, color::Color, Cb};
use ambient_network::{client::ClientState, server::RpcArgs as ServerRpcArgs};
use ambient_renderer::{RenderTarget, Renderer};
use ambient_rpc::RpcRegistry;
use ambient_shared_types::{ModifiersState, VirtualKeyCode};
use ambient_std::line_uid;
use ambient_ui_native::{
    fit_horizontal, height, space_between_items, width, Button, ButtonStyle, Dropdown, Fit,
    FlowColumn, FlowRow, Image, Text, UIExt,
};
use glam::Vec3;

type GetDebuggerState =
    Cb<dyn Fn(&mut dyn FnMut(&mut Renderer, &RenderTarget, &mut World)) + Sync + Send>;

pub async fn rpc_dump_world_hierarchy(args: ServerRpcArgs, _: ()) -> Option<String> {
    let mut res = Vec::new();
    let mut state = args.state.lock();
    let world = state.get_player_world_mut(&args.user_id)?;
    dump_world_hierarchy(world, &mut res);
    Some(String::from_utf8(res).unwrap())
}

pub fn register_server_rpcs(reg: &mut RpcRegistry<ServerRpcArgs>) {
    reg.register(rpc_dump_world_hierarchy);
}

fn dump_to_user(_assets: &AssetCache, _label: &'static str, s: String) {
    #[cfg(target_os = "unknown")]
    {
        ambient_sys::clipboard::set_background(s, move |res| match res {
            Ok(()) => tracing::info!("Wrote {_label} to clipboard"),
            Err(err) => tracing::error!("Failed to write {_label} to clipboard: {err:?}"),
        })
    }
    #[cfg(not(target_os = "unknown"))]
    {
        let rt = ambient_native_std::asset_cache::SyncAssetKeyExt::get(
            &ambient_core::RuntimeKey,
            _assets,
        );
        let cache_dir = ambient_native_std::asset_cache::SyncAssetKeyExt::get(
            &ambient_native_std::download_asset::AssetsCacheDir,
            _assets,
        );

        rt.spawn(async move {
            let path = cache_dir.join(_label);

            ambient_sys::fs::create_dir_all(cache_dir)
                .await
                .expect("Failed to create tmp dir");

            ambient_sys::fs::write(&path, s)
                .await
                .expect("Failed to write to file");

            tracing::info!("Dumped renderer to {:?}", path);
        });
    }
}

#[derive(Debug, Clone)]
struct Measurement {
    min: Duration,
    max: Duration,
    current: Duration,
    sum: Duration,
    count: usize,
}

impl Measurement {
    fn new() -> Self {
        Self {
            min: Duration::MAX,
            max: Duration::ZERO,
            current: Duration::ZERO,
            sum: Duration::ZERO,
            count: 0,
        }
    }

    fn apply(&mut self, value: Duration) {
        self.min = self.min.min(value);
        self.max = self.max.max(value);
        self.current = value;
        self.sum += value;
        self.count += 1;
    }

    fn avg(&self) -> Duration {
        self.sum.checked_div(self.count as u32).unwrap_or_default()
    }
}

impl std::fmt::Display for Measurement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("min: ")?;
        Debug::fmt(&self.min, f)?;
        f.write_str(" max: ")?;
        Debug::fmt(&self.max, f)?;
        f.write_str(" avg: ")?;
        Debug::fmt(&self.avg(), f)?;
        f.write_str(" current: ")?;
        Debug::fmt(&self.current, f)?;

        Ok(())
    }
}

#[element_component]
pub fn AppStatsView(hooks: &mut Hooks) -> Element {
    let (measurements, set_measurements) =
        use_state(hooks, (Measurement::new(), Measurement::new()));

    use_frame(hooks, move |w| {
        let samples = w.resource(performance_samples());

        let mut frame_time = Measurement::new();
        let mut external_time = Measurement::new();
        assert!(samples.len() <= 128);

        for sample in samples {
            frame_time.apply(sample.frame_time);
            external_time.apply(sample.external_time);
        }

        set_measurements((frame_time, external_time))
    });

    FlowColumn::el(vec![
        Text::el(format!("Frame time    {:<8.1}", measurements.0)),
        Text::el(format!("External time {:<8.1}", measurements.1)),
    ])
}

#[element_component]
pub fn Debugger(hooks: &mut Hooks, get_state: GetDebuggerState) -> Element {
    let (show_shadows, set_show_shadows) = use_state(hooks, false);
    let (client_state, _) = consume_context::<ClientState>(hooks).unwrap();

    FlowColumn::el([
        FlowRow(vec![
            AppStatsView.el(),
            Button::new("Dump Client World", {
                let get_state = get_state.clone();
                move |_world| {
                    get_state(&mut |_, _, world| {
                        dump_world_hierarchy_to_user(world);
                    });
                }
            })
            .hotkey_modifier(ModifiersState::SHIFT)
            .hotkey(VirtualKeyCode::F1)
            .style(ButtonStyle::Flat)
            .el(),
            Button::new("Dump Server World", {
                let client_state = client_state;
                move |world| {
                    let assets = world.resource(asset_cache()).clone();
                    let client_state = client_state.clone();
                    world.resource(runtime()).clone().spawn(async move {
                        if let Ok(Some(res)) = client_state.rpc(rpc_dump_world_hierarchy, ()).await
                        {
                            dump_to_user(&assets, "server_hierarchy.yml", res);
                        }
                    });
                }
            })
            .hotkey_modifier(ModifiersState::SHIFT)
            .hotkey(VirtualKeyCode::F2)
            .style(ButtonStyle::Flat)
            .el(),
            Button::new("Dump Client Renderer", {
                let get_state = get_state.clone();
                move |world| {
                    let assets = world.resource(asset_cache());
                    let mut s = Vec::new();
                    tracing::info!("Dumping renderer");
                    get_state(&mut |renderer, _, _| renderer.dump(&mut s));
                    dump_to_user(assets, "renderer.txt", String::from_utf8(s).unwrap());
                }
            })
            .hotkey_modifier(ModifiersState::SHIFT)
            .hotkey(VirtualKeyCode::F3)
            .style(ButtonStyle::Flat)
            .el(),
            Button::new("Show Shadow Frustums", {
                let get_state = get_state.clone();
                move |_| {
                    get_state(&mut |_, _, world| {
                        let gizmos = world.resource(gizmos());
                        let mut g = gizmos.scope(line_uid!());
                        let cascades = 5;
                        for (i, cam) in shadow_cameras_from_world(
                            world,
                            cascades,
                            1024,
                            Vec3::ONE.normalize(),
                            main_scene(),
                            world.resource_opt(local_user_id()),
                        )
                        .into_iter()
                        .enumerate()
                        {
                            for line in cam.world_space_frustum_lines() {
                                g.draw(GizmoPrimitive::line(line.0, line.1, 1.).with_color(
                                    Color::hsl(360. * i as f32 / cascades as f32, 1.0, 0.5).into(),
                                ));
                            }
                        }
                    })
                }
            })
            .hotkey_modifier(ModifiersState::SHIFT)
            .hotkey(VirtualKeyCode::F4)
            .style(ButtonStyle::Flat)
            .el(),
            Button::new("Show World Boundings", {
                let get_state = get_state.clone();
                move |_| {
                    get_state(&mut |_, _, world| {
                        let gizmos = world.resource(gizmos());
                        let mut g = gizmos.scope(line_uid!());
                        for (_, (bounding,)) in query((world_bounding_sphere(),)).iter(world, None)
                        {
                            g.draw(
                                GizmoPrimitive::sphere(bounding.center, bounding.radius)
                                    .with_color(Vec3::ONE),
                            );
                        }
                    });
                }
            })
            .hotkey_modifier(ModifiersState::SHIFT)
            .hotkey(VirtualKeyCode::F5)
            .style(ButtonStyle::Flat)
            .el(),
            Button::new("Show Shadow Maps", {
                move |_| {
                    set_show_shadows(!show_shadows);
                }
            })
            .hotkey_modifier(ModifiersState::SHIFT)
            .hotkey(VirtualKeyCode::F6)
            .style(ButtonStyle::Flat)
            .el(),
            ShaderDebug {
                get_state: get_state.clone(),
            }
            .el(),
            // Button::new("Dump Internal UI World", {
            //     move |world| {
            //         dump_world_hierarchy_to_tmp_file(world);
            //     }
            // })
            // .style(ButtonStyle::Flat)
            // .el(),
        ])
        .el()
        .with(space_between_items(), 5.),
        if show_shadows {
            ShadowMapsViz {
                get_state: get_state.clone(),
            }
            .el()
        } else {
            Element::new()
        },
    ])
    .with_background(Color::rgba(0., 0., 0., 1.).into())
    .with(fit_horizontal(), Fit::Parent)
}

#[element_component]
fn ShadowMapsViz(hooks: &mut Hooks, get_state: GetDebuggerState) -> Element {
    let (shadow_cascades, _) = use_state_with(hooks, |_| {
        let mut n_cascades = 0;
        get_state(&mut |renderer, _, _| {
            n_cascades = renderer.config.shadow_cascades;
        });
        n_cascades
    });
    FlowRow::el(
        (0..shadow_cascades)
            .map(|i| {
                ShadowMapViz {
                    get_state: get_state.clone(),
                    cascade: i,
                }
                .el()
            })
            .collect::<Vec<_>>(),
    )
    .with(space_between_items(), 5.)
    .with_background(Color::rgb(0.0, 0., 0.3).into())
}

#[element_component]
fn ShadowMapViz(hooks: &mut Hooks, get_state: GetDebuggerState, cascade: u32) -> Element {
    let (texture, _) = use_state_with(hooks, |_| {
        let mut tex = None;
        get_state(&mut |renderer, _, _| {
            tex = Some(renderer.shadows.as_ref().map(|x| {
                Arc::new(x.shadow_texture.create_view(&wgpu::TextureViewDescriptor {
                    base_array_layer: cascade,
                    array_layer_count: Some(1),
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    ..Default::default()
                }))
            }));
        });
        tex.unwrap()
    });
    Image { texture }
        .el()
        .with(width(), 200.)
        .with(height(), 200.)
}

#[element_component]
fn ShaderDebug(hooks: &mut Hooks, get_state: GetDebuggerState) -> Element {
    let (show, set_show) = use_state(hooks, false);

    let (_, upd) = use_state(hooks, ());

    let mut params = Default::default();
    get_state(&mut |renderer, _, _| {
        params = renderer.shader_debug_params;
    });
    let metallic_roughness = params.metallic_roughness;
    let normals = params.normals;
    let shading = params.shading;

    Dropdown {
        content: Button::new("Shader Debug", move |_| set_show(!show))
            .toggled(show)
            .el(),
        dropdown: FlowColumn::el([
            Button::new("Show metallic roughness", {
                let get_state = get_state.clone();
                let upd = upd.clone();
                move |_| {
                    get_state(&mut |renderer, _, _| {
                        renderer.shader_debug_params.metallic_roughness =
                            (1.0 - metallic_roughness).round();
                    });
                    upd(())
                }
            })
            .toggled(metallic_roughness > 0.0)
            .el(),
            Button::new("Show normals", {
                let get_state = get_state.clone();
                let upd = upd.clone();
                move |_| {
                    get_state(&mut |renderer, _, _| {
                        renderer.shader_debug_params.normals = (1.0 - normals).round();
                    });
                    upd(())
                }
            })
            .toggled(normals > 0.0)
            .el(),
            Button::new("Disable shading", {
                let get_state = get_state.clone();
                let upd = upd.clone();
                move |_| {
                    get_state(&mut |renderer, _, _| {
                        renderer.shader_debug_params.shading = (1.0 - shading).round();
                    });
                    upd(())
                }
            })
            .toggled(shading > 0.0)
            .el(),
        ]),
        show,
    }
    .el()
}
