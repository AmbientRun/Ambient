use ambient_api::{
    components::core::{
        app::ui_scene,
        camera::{orthographic_bottom, orthographic_left, orthographic_right, orthographic_top},
        game_objects::player_camera,
    },
    concepts::make_orthographic_camera,
    prelude::*,
};
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_guest_bridge::{
    components::ui::{
        background_color, border_color, border_thickness, height, space_between_items, width,
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
            .set(border_thickness(), 10.),
    ])
    .set(space_between_items(), 10.)
}

#[main]
pub async fn main() -> EventResult {
    Entity::new()
        .with_merge(make_orthographic_camera())
        .with(orthographic_left(), -300.)
        .with(orthographic_right(), 300.)
        .with(orthographic_top(), -300.)
        .with(orthographic_bottom(), 300.)
        .with_default(player_camera())
        .with_default(ui_scene())
        .spawn();

    let mut tree = App.el().spawn_tree();
    on(ambient_api::event::FRAME, move |_| {
        tree.update(&mut World);
        EventOk
    });

    EventOk
}
