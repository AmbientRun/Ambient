use ambient_api::{
    components::core::{app::ui_scene, game_objects::player_camera},
    concepts::make_orthographic_camera,
    prelude::*,
};
use ambient_element::{element_component, Element, ElementComponentExt, Group, Hooks};
use ambient_guest_bridge::{
    components::{
        camera::orthographic_from_window,
        ui::{
            align_horizontal_center, align_horizontal_end, align_vertical_center,
            align_vertical_end, fit_horizontal_children, fit_horizontal_none,
            fit_vertical_children, fit_vertical_none, font_size, height, space_between_items,
            width,
        },
    },
    ecs::World,
};
use ambient_ui_components::{
    layout::{FlowColumn, FlowRow},
    text::Text,
    UIExt2,
};

#[element_component]
fn App(_hooks: &mut Hooks) -> Element {
    let background = |e| {
        FlowRow(vec![e])
            .el()
            .with_background(vec4(1., 1., 1., 0.02))
    };
    Group(vec![FlowColumn(vec![
        FlowRow(vec![Text::el("Basic")])
            .el()
            .with_background(vec4(0.1, 0.1, 0.1, 1.))
            .set_default(fit_vertical_children())
            .set_default(fit_horizontal_children())
            .with_padding_even(10.),
        FlowRow(vec![
            Text::el("Spacing"),
            Text::el("between"),
            Text::el("items"),
        ])
        .el()
        .with_background(vec4(0.1, 0.1, 0.1, 1.))
        .set_default(fit_vertical_children())
        .set_default(fit_horizontal_children())
        .with_padding_even(10.)
        .set(space_between_items(), 50.),
        FlowRow(vec![Text::el("Break"), Text::el("line")])
            .el()
            .with_background(vec4(0.1, 0.1, 0.1, 1.))
            .set_default(fit_vertical_children())
            .set_default(fit_horizontal_none())
            .set(width(), 50.)
            .with_padding_even(10.),
        FlowRow(vec![
            background(Text::el("Align")),
            background(Text::el("Center").set(font_size(), 30.)),
        ])
        .el()
        .with_background(vec4(0.1, 0.1, 0.1, 1.))
        .set_default(fit_vertical_none())
        .set_default(fit_horizontal_none())
        .set_default(align_horizontal_center())
        .set_default(align_vertical_center())
        .set(width(), 200.)
        .set(height(), 70.)
        .with_padding_even(10.)
        .set(space_between_items(), 5.),
        FlowRow(vec![
            background(Text::el("Align")),
            background(Text::el("End").set(font_size(), 30.)),
        ])
        .el()
        .with_background(vec4(0.1, 0.1, 0.1, 1.))
        .set_default(fit_vertical_none())
        .set_default(fit_horizontal_none())
        .set_default(align_horizontal_end())
        .set_default(align_vertical_end())
        .set(width(), 200.)
        .set(height(), 70.)
        .with_padding_even(10.)
        .set(space_between_items(), 5.),
    ])
    .el()
    .set(space_between_items(), 5.)
    .with_padding_even(5.)])
    .el()
}

#[main]
pub async fn main() -> EventResult {
    Entity::new()
        .with_merge(make_orthographic_camera())
        .with_default(orthographic_from_window())
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
