use std::sync::Arc;

use ambient_core::window::{
    cursor_position, set_cursor, window_logical_size, window_physical_size, window_scale_factor,
};
use ambient_debugger::Debugger;
use ambient_ecs::{generated::messages, EntityId};
use ambient_ecs_editor::{ECSEditor, InspectableAsyncWorld};
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_network::client::{ClientState, GameClientRenderTarget, GameClientWorld};
use ambient_shared_types::CursorIcon;
use ambient_ui_native::{
    cb, docking_impl, padding, width, Borders, Button, Dock, MeasureSize, ScrollArea,
    ScrollAreaSizing, UIExt, STREET,
};
use glam::{uvec2, vec4, Vec2};

#[element_component]
pub fn GameView(hooks: &mut Hooks, show_debug: bool) -> Element {
    let (client_state, _) = hooks.consume_context::<ClientState>().unwrap();
    let (render_target, _) = hooks.consume_context::<GameClientRenderTarget>().unwrap();

    let (show_ecs, set_show_ecs) = hooks.use_state(true);
    let (ecs_size, set_ecs_size) = hooks.use_state(Vec2::ZERO);
    let (debugger_size, set_debugger_size) = hooks.use_state(Vec2::ZERO);

    let (w, set_w) = hooks.use_state(300.0);
    let (w_memory, set_w_memory) = hooks.use_state(0.0);
    let (mouse_on_edge, set_mouse_on_edge) = hooks.use_state(false);
    let (should_track_resize, set_should_track_resize) = hooks.use_state(false);

    hooks.use_runtime_message::<messages::WindowMouseInput>({
        move |_world, event| {
            let pressed = event.pressed;
            if pressed && mouse_on_edge {
                set_should_track_resize(true);
            } else {
                set_should_track_resize(false);
            }
        }
    });

    hooks.use_frame({
        let state = client_state.clone();
        let render_target = render_target.clone();
        let set_w = set_w.clone();
        let set_w_memory = set_w_memory.clone();
        move |world| {
            let mut state = state.game_state.lock();

            let scale_factor = *world.resource(window_scale_factor());
            let mut mouse_pos = *world.resource(cursor_position());
            if (w - mouse_pos.x).abs() < 5.0 && show_debug {
                set_cursor(world, CursorIcon::ColResize.into());
                set_mouse_on_edge(true);
            } else {
                set_cursor(world, CursorIcon::Default.into());
                set_mouse_on_edge(false);
            }
            if should_track_resize {
                set_w(mouse_pos.x);
                set_w_memory(mouse_pos.x);
            }
            mouse_pos.x -= ecs_size.x;
            mouse_pos.y -= debugger_size.y;

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
                Dock::el([
                    Button::new(if show_ecs { "\u{f137}" } else { "\u{f138}" }, move |_| {
                        set_show_ecs(!show_ecs)
                    })
                    .style(ambient_ui_native::ButtonStyle::Flat)
                    .toggled(show_ecs)
                    .el(),
                    if show_ecs {
                        if w_memory != 0.0 {
                            set_w(w_memory)
                        } else {
                            set_w(300.0)
                        };
                        ScrollArea::el(
                            ScrollAreaSizing::FitParentWidth,
                            ECSEditor {
                                world: Arc::new(InspectableAsyncWorld(cb({
                                    let client_state = client_state.clone();
                                    move |res| {
                                        let client_state = client_state.game_state.lock();
                                        res(&client_state.world)
                                    }
                                }))),
                            }
                            .el()
                            .memoize_subtree(client_state.uid),
                        )
                    } else {
                        set_w(0.0);
                        Element::new()
                    },
                ])
                .with(width(), w)
                .with(docking_impl(), ambient_layout::Docking::Left)
                .with_background(vec4(0., 0., 0., 1.))
                .with(padding(), Borders::even(STREET).into()),
                set_ecs_size,
            )
        } else {
            Element::new()
        },
        if show_debug {
            MeasureSize::el(
                Debugger {
                    get_state: cb(move |cb| {
                        let mut game_state = client_state.game_state.lock();
                        let game_state = &mut *game_state;
                        cb(
                            &mut game_state.renderer,
                            &render_target.0,
                            &mut game_state.world,
                        );
                    }),
                }
                .el(),
                set_debugger_size,
            )
            .with(docking_impl(), ambient_layout::Docking::Top)
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
