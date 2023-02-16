use std::{num::NonZeroU32, sync::Arc};

use glam::Vec3;
use kiwi_core::{
    asset_cache,
    bounding::world_bounding_sphere,
    camera::shadow_cameras_from_world,
    hierarchy::{dump_world_hierarchy, dump_world_hierarchy_to_tmp_file},
    main_scene, runtime,
};
use kiwi_ecs::{query, World};
use kiwi_ecs_editor::ECSEditor;
use kiwi_element::{element_component, Element, ElementComponentExt, Hooks};
use kiwi_gizmos::{gizmos, GizmoPrimitive};
use kiwi_network::client::{GameClient, GameRpcArgs};
use kiwi_renderer::{RenderTarget, Renderer};
use kiwi_rpc::RpcRegistry;
use kiwi_std::{asset_cache::SyncAssetKeyExt, cb, color::Color, download_asset::AssetsCacheDir, line_hash, Cb};
use kiwi_ui::{
    fit_horizontal, height, space_between_items, width, Button, ButtonStyle, Dropdown, Fit, FlowColumn, FlowRow, Image, UIExt,
    VirtualKeyCode,
};
use winit::event::ModifiersState;

type GetDebuggerState = Cb<dyn Fn(&mut dyn FnMut(&mut Renderer, &RenderTarget, &mut World)) + Sync + Send>;

pub async fn rpc_dump_world_hierarchy(args: GameRpcArgs, _: ()) -> Option<String> {
    let mut res = Vec::new();
    let mut state = args.state.lock();
    let world = state.get_player_world_mut(&args.user_id)?;
    dump_world_hierarchy(world, &mut res);
    Some(String::from_utf8(res).unwrap())
}

pub fn register_rpcs(reg: &mut RpcRegistry<GameRpcArgs>) {
    reg.register(rpc_dump_world_hierarchy);
}

#[element_component]
pub fn Debugger(hooks: &mut Hooks, get_state: GetDebuggerState) -> Element {
    let (show_shadows, set_show_shadows) = hooks.use_state(false);
    let (show_ecs, set_show_ecs) = hooks.use_state(false);
    let (game_client, _) = hooks.consume_context::<GameClient>().unwrap();
    FlowColumn::el([
        FlowRow(vec![
            Button::new("Show entities", {
                move |_| {
                    set_show_ecs(!show_ecs);
                }
            })
            .toggled(show_ecs)
            .hotkey_modifier(ModifiersState::SHIFT)
            .hotkey(VirtualKeyCode::F1)
            .style(ButtonStyle::Flat)
            .el(),
            Button::new("Dump Client World", {
                let get_state = get_state.clone();
                move |_world| {
                    get_state(&mut |_, _, world| dump_world_hierarchy_to_tmp_file(world));
                }
            })
            .hotkey_modifier(ModifiersState::SHIFT)
            .hotkey(VirtualKeyCode::F1)
            .style(ButtonStyle::Flat)
            .el(),
            Button::new("Dump Server World", {
                let game_client = game_client;
                move |world| {
                    let game_client = game_client.clone();
                    let cache_dir = AssetsCacheDir.get(world.resource(asset_cache()));
                    world.resource(runtime()).clone().spawn(async move {
                        if let Ok(Some(res)) = game_client.rpc(rpc_dump_world_hierarchy, ()).await {
                            std::fs::create_dir_all(&cache_dir).ok();
                            let path = cache_dir.join("server_hierarchy.yml");
                            std::fs::write(&path, res).ok();
                            log::info!("Wrote {:?}", path);
                        }
                    });
                }
            })
            .hotkey_modifier(ModifiersState::SHIFT)
            .hotkey(VirtualKeyCode::F6)
            .style(ButtonStyle::Flat)
            .el(),
            Button::new("Dump Client Renderer", {
                let get_state = get_state.clone();
                move |world| {
                    let cache_dir = AssetsCacheDir.get(world.resource(asset_cache()));
                    std::fs::create_dir_all(&cache_dir).ok();
                    let path = cache_dir.join("renderer.txt");
                    std::fs::create_dir_all(cache_dir).expect("Failed to create tmp dir");
                    let mut f = std::fs::File::create(path).expect("Unable to create file");
                    get_state(&mut |renderer, _, _| renderer.dump(&mut f));
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
                        let mut g = gizmos.scope(line_hash!());
                        let cascades = 5;
                        for (i, cam) in
                            shadow_cameras_from_world(world, cascades, 1024, Vec3::ONE.normalize(), main_scene()).into_iter().enumerate()
                        {
                            for line in cam.world_space_frustum_lines() {
                                g.draw(
                                    GizmoPrimitive::line(line.0, line.1, 1.)
                                        .with_color(Color::hsl(360. * i as f32 / cascades as f32, 1.0, 0.5).into()),
                                );
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
                        let mut g = gizmos.scope(line_hash!());
                        for (_, (bounding,)) in query((world_bounding_sphere(),)).iter(world, None) {
                            g.draw(GizmoPrimitive::sphere(bounding.center, bounding.radius).with_color(Vec3::ONE));
                        }
                    });
                }
            })
            .hotkey_modifier(ModifiersState::SHIFT)
            .hotkey(VirtualKeyCode::F5)
            .style(ButtonStyle::Flat)
            .el(),
            ShaderDebug { get_state: get_state.clone() }.el(),
            Button::new("Show Shadow Maps", {
                move |_| {
                    set_show_shadows(!show_shadows);
                }
            })
            .style(ButtonStyle::Flat)
            .el(),
        ])
        .el()
        .set(space_between_items(), 5.),
        if show_shadows { ShadowMapsViz { get_state: get_state.clone() }.el() } else { Element::new() },
        if show_ecs {
            ECSEditor { get_world: cb(move |res| get_state(&mut move |_, _, world| res(world))), on_change: cb(|_, _| {}) }
                .el()
                .set(height(), 200.)
        } else {
            Element::new()
        },
    ])
    .with_background(Color::rgba(0., 0., 0., 1.))
    .set(fit_horizontal(), Fit::Parent)
}

#[element_component]
fn ShadowMapsViz(hooks: &mut Hooks, get_state: GetDebuggerState) -> Element {
    let (shadow_cascades, _) = hooks.use_state_with(|_| {
        let mut n_cascades = 0;
        get_state(&mut |renderer, _, _| {
            n_cascades = renderer.config.shadow_cascades;
        });
        n_cascades
    });
    FlowRow::el((0..shadow_cascades).map(|i| ShadowMapViz { get_state: get_state.clone(), cascade: i }.el()).collect::<Vec<_>>())
        .set(space_between_items(), 5.)
        .with_background(Color::rgb(0.0, 0., 0.3))
}

#[element_component]
fn ShadowMapViz(hooks: &mut Hooks, get_state: GetDebuggerState, cascade: u32) -> Element {
    let (texture, _) = hooks.use_state_with(|_| {
        let mut tex = None;
        get_state(&mut |renderer, _, _| {
            tex = Some(renderer.shadows.as_ref().map(|x| {
                Arc::new(x.shadow_texture.create_view(&wgpu::TextureViewDescriptor {
                    base_array_layer: cascade,
                    array_layer_count: NonZeroU32::new(1),
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    ..Default::default()
                }))
            }));
        });
        tex.unwrap()
    });
    Image { texture }.el().set(width(), 200.).set(height(), 200.)
}

#[element_component]
fn ShaderDebug(hooks: &mut Hooks, get_state: GetDebuggerState) -> Element {
    let (show, set_show) = hooks.use_state(false);

    let (_, upd) = hooks.use_state(());

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
            .hotkey(VirtualKeyCode::F7)
            .hotkey_modifier(ModifiersState::SHIFT)
            .el(),
        dropdown: FlowColumn::el([
            Button::new("Show metallic roughness", {
                let get_state = get_state.clone();
                let upd = upd.clone();
                move |_| {
                    get_state(&mut |renderer, _, _| {
                        renderer.shader_debug_params.metallic_roughness = (1.0 - metallic_roughness).round();
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
