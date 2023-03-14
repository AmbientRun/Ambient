#![cfg(feature = "server")]
use ambient_api::{
    components::core::app::ui_scene, concepts::make_orthographic_camera, prelude::*,
};
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_guest_bridge::{
    components::{
        camera::orthographic_from_window,
        player::{player, user_id},
        ui::{
            background_color, border_color, border_radius, border_thickness, height,
            space_between_items, width,
        },
    },
    ecs::World,
};
use ambient_ui_components::{layout::FlowColumn, Rectangle};

#[element_component]
fn App(_hooks: &mut Hooks) -> Element {
    FlowColumn::el([
        Rectangle.el(),
        Rectangle
            .el()
            .set(width(), 150.)
            .set(height(), 50.)
            .set(background_color(), vec4(1., 0., 0., 1.))
            .set(border_color(), vec4(0., 1., 0., 1.))
            .set(border_thickness(), 10.)
            .set(border_radius(), vec4(20., 10., 5., 0.)),
    ])
    .set(space_between_items(), 10.)
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
