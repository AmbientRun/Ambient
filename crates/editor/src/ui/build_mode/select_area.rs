use std::sync::Arc;

use kiwi_core::{
    mouse_position, on_window_event, runtime, transform::{get_world_position, translation}, window_logical_size, window_scale_factor
};
use kiwi_ecs::World;
use kiwi_element::{Element, ElementComponent, ElementComponentExt, Hooks};
use kiwi_input::MouseButton;
use kiwi_network::{client::GameClient, log_network_result};
use kiwi_std::{color::Color, math::interpolate};
use kiwi_ui::{
    layout::{height, width}, UIBase, UIExt
};
use glam::{vec2, vec3, Vec2, Vec3Swizzles};
use winit::event::{ElementState, WindowEvent};

use crate::{
    intents::SelectMode, rpc::{rpc_select, SelectMethod}
};

#[derive(Debug, Clone)]
/// Handles the server communication for selecting objects
pub struct SelectArea;
impl ElementComponent for SelectArea {
    fn render(self: Box<Self>, _world: &mut World, hooks: &mut Hooks) -> Element {
        let (dragging, set_dragging) = hooks.use_state(None);
        let (area_offset, set_area_offset) = hooks.use_state(Vec2::ZERO);
        let (mouse_pos, set_mouse_pos) = hooks.use_state(Vec2::ZERO);
        let (game_client, _) = hooks.consume_context::<GameClient>().unwrap();
        let (select_mode, _) = hooks.consume_context::<SelectMode>().unwrap();
        let is_clicking = hooks.use_ref_with(|| false);

        let client = game_client.clone();
        hooks.use_spawn(move |_| {
            Box::new(move |w| {
                w.resource(runtime()).spawn(async move {
                    log_network_result!(client.rpc(rpc_select, (SelectMethod::Manual(Default::default()), SelectMode::Clear)).await);
                });
            })
        });

        UIBase
            .el()
            .on_mouse_down(closure!(clone set_dragging, clone is_clicking, |world, id, button| {
                if button != MouseButton::Left {
                    return;
                }

                let area_offset = get_world_position(world, id).unwrap().xy();
                let scl = *world.resource(window_scale_factor()) as f32;
                set_dragging(Some(*world.resource(mouse_position()) / scl));
                set_area_offset(area_offset);
                tracing::info!("Set is_clicking to true");
                *is_clicking.lock() = true;
            }))
            .listener(
                on_window_event(),
                Arc::new(move |world, _, event| {
                    let scl = *world.resource(window_scale_factor()) as f32;
                    match event {
                        WindowEvent::CursorMoved { position, .. } => {
                            set_mouse_pos(vec2(position.x as f32, position.y as f32) / scl);
                        }
                        WindowEvent::MouseInput { state, .. } => {
                            if state == &ElementState::Released {
                                let mut is_clicking = is_clicking.lock();
                                if !*is_clicking {
                                    return;
                                }

                                *is_clicking = false;

                                tracing::info!("Released selection click");
                                set_dragging(None);

                                let screen_size = world.resource(window_logical_size()).as_vec2();

                                if let Some(dragging) = dragging {
                                    let game_client = game_client.clone();
                                    let min_x = dragging.x.min(mouse_pos.x);
                                    let max_x = dragging.x.max(mouse_pos.x);
                                    let min_y = dragging.y.min(mouse_pos.y);
                                    let max_y = dragging.y.max(mouse_pos.y);
                                    let size = vec2(max_x - min_x, max_y - min_y);
                                    if size.x > 5. || size.y > 5. {
                                        let frustum = {
                                            let state = game_client.game_state.lock();
                                            let get_corner = |p, z| {
                                                let clip_pos = interpolate(p, Vec2::ZERO, screen_size, vec2(-1., 1.), vec2(1., -1.));
                                                state.clip_to_world_space(clip_pos.extend(z))
                                            };
                                            [
                                                // Order: Bottom [ Left (Back, Front), Right (Back, Front) ] - Top [ Left (Back, Front), Right (Back, Front) ]
                                                get_corner(vec2(min_x, min_y), 1.),
                                                get_corner(vec2(min_x, min_y), 0.001),
                                                get_corner(vec2(max_x, min_y), 1.),
                                                get_corner(vec2(max_x, min_y), 0.001),
                                                get_corner(vec2(min_x, max_y), 1.),
                                                get_corner(vec2(min_x, max_y), 0.001),
                                                get_corner(vec2(max_x, max_y), 1.),
                                                get_corner(vec2(max_x, max_y), 0.001),
                                            ]
                                        };
                                        world.resource(runtime()).clone().spawn(async move {
                                            log_network_result!(
                                                game_client.rpc(rpc_select, (SelectMethod::Frustum(frustum), select_mode)).await
                                            );
                                        });
                                        return;
                                    }
                                }

                                let ray = {
                                    let state = game_client.game_state.lock();
                                    let p = interpolate(mouse_pos, Vec2::ZERO, screen_size, vec2(-1., 1.), vec2(1., -1.));
                                    state.screen_ray(p)
                                };

                                let game_client = game_client.clone();
                                world.resource(runtime()).clone().spawn(async move {
                                    log_network_result!(game_client.rpc(rpc_select, (SelectMethod::Ray(ray), select_mode)).await);
                                });
                            }
                        }
                        _ => {}
                    }
                }),
            )
            .children(vec![if let Some(dragging) = dragging {
                let min_x = dragging.x.min(mouse_pos.x);
                let max_x = dragging.x.max(mouse_pos.x);
                let min_y = dragging.y.min(mouse_pos.y);
                let max_y = dragging.y.max(mouse_pos.y);
                UIBase
                    .el()
                    .with_background(Color::rgba(0., 0., 1., 0.3))
                    .set(translation(), vec3(min_x, min_y, -0.05) - area_offset.extend(0.))
                    .set(width(), max_x - min_x)
                    .set(height(), max_y - min_y)
            } else {
                Element::new()
            }])
    }
}
