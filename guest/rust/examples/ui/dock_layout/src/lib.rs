use ambient_api::{
    components::core::{app::ui_scene, game_objects::player_camera},
    concepts::make_orthographic_camera,
    prelude::*,
};
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_guest_bridge::{
    components::{
        camera::orthographic_from_window,
        player::{player, user_id},
        transform::translation,
        ui::{docking_bottom, docking_left, fit_horizontal_none, fit_vertical_none, height, width},
    },
    ecs::World,
};
use ambient_ui_components::{
    layout::{Dock, FlowRow},
    text::Text,
    UIExt,
};

#[element_component]
fn App(_hooks: &mut Hooks) -> Element {
    let background = |e| {
        FlowRow(vec![e])
            .el()
            .with_background(vec4(1., 1., 1., 0.02).into())
            .set_default(fit_vertical_none())
            .set_default(fit_horizontal_none())
    };
    Dock(vec![
        background(Text::el("First"))
            .set(height(), 30.)
            .with_margin_even(10.),
        background(Text::el("Second bottom"))
            .set_default(docking_bottom())
            .set(height(), 50.)
            .with_margin_even(10.),
        background(Text::el("Third left"))
            .set_default(docking_left())
            .set(width(), 70.),
        Dock(vec![background(Text::el("Fourth padding"))])
            .el()
            .with_padding_even(10.)
            .set(height(), 70.)
            .with_background(vec4(1., 1., 1., 0.02).into()),
        background(Text::el("Fill remainder")).with_margin_even(30.),
    ])
    .el()
    .with_background(vec4(1., 1., 1., 0.02).into())
    .set(translation(), vec3(10., 10., 0.))
    .set(width(), 500.)
    .set(height(), 500.)
}

#[main]
pub async fn main() -> EventResult {
    spawn_query((player(), user_id())).bind(move |players| {
        for (id, _) in players {
            entity::add_components(
                id,
                Entity::new()
                    .with_merge(make_orthographic_camera())
                    .with(orthographic_from_window(), id)
                    .with_default(player_camera())
                    .with_default(ui_scene()),
            );
        }
    });

    let mut tree = App.el().spawn_tree();
    on(ambient_api::event::FRAME, move |_| {
        tree.update(&mut World);
        EventOk
    });

    EventOk
}
