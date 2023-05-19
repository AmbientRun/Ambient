use ambient_core::window::{
    cursor_position, window_logical_size, window_physical_size, window_scale_factor,
};
use ambient_ecs::EntityId;
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_network::client::{GameClient, GameClientRenderTarget, GameClientWorld};
use ambient_ui_native::{
    docking, padding, Borders, Button, Dock, Docking, FlowColumn, MeasureSize, UIExt, STREET,
};
use glam::{uvec2, vec4, Vec2};

#[element_component]
pub fn GameView(hooks: &mut Hooks, show_debug: bool) -> Element {
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
                    // if show_ecs {
                    //     ScrollArea::el(
                    //         ScrollAreaSizing::FitChildrenWidth,
                    //         ECSEditor {
                    //             world: Arc::new(InspectableAsyncWorld(cb({
                    //                 let state = state.clone();
                    //                 move |res| {
                    //                     let state = state.game_state.lock();
                    //                     res(&state.world)
                    //                 }
                    //             }))),
                    //         }
                    //         .el()
                    //         .memoize_subtree(state.uid),
                    //     )
                    // } else {
                    //     Element::new()
                    // },
                ])
                .with(docking(), Docking::Left)
                .with_background(vec4(0., 0., 1., 1.))
                .with(padding(), Borders::even(STREET).into()),
                set_ecs_size,
            )
        } else {
            Element::new()
        },
        // if show_debug {
        //     Debugger {
        //         get_state: cb(move |cb| {
        //             let mut game_state = state.game_state.lock();
        //             let game_state = &mut *game_state;
        //             cb(
        //                 &mut game_state.renderer,
        //                 &render_target.0,
        //                 &mut game_state.world,
        //             );
        //         }),
        //     }
        //     .el()
        //     .with(docking(), ambient_layout::Docking::Bottom)
        //     .with(padding(), Borders::even(STREET).into())
        // } else {
        //     Element::new()
        // },
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
