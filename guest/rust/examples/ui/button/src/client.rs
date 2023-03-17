use ambient_api::{
    components::core::app::ui_scene, concepts::make_orthographic_camera, prelude::*,
};
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_guest_bridge::{
    components::{
        camera::orthographic_from_window,
        layout::space_between_items,
        player::{player, user_id},
        transform::translation,
    },
    ecs::World,
};
use ambient_ui_components::{
    button::{Button, ButtonStyle},
    default_theme::STREET,
    layout::{FlowColumn, FlowRow},
    text::Text,
    UIExt,
};

#[element_component]
fn App(_hooks: &mut Hooks) -> Element {
    let card_inner = |text| {
        FlowRow(vec![Text::el(text)])
            .el()
            .with_background(vec4(0.3, 0.3, 0.3, 1.))
            .with_padding_even(20.)
    };

    FlowRow(vec![
        FlowColumn(vec![
            Button::new("Regular", |_| {}).el(),
            Button::new("Primary", |_| {})
                .style(ButtonStyle::Primary)
                .tooltip(Text::el("Tooltip"))
                .el(),
            Button::new("Flat", |_| {}).style(ButtonStyle::Flat).el(),
            Button::new(card_inner("Card"), |_| {})
                .style(ButtonStyle::Card)
                .el(),
            Button::new("Inline", |_| {})
                .style(ButtonStyle::Inline)
                .el(),
        ])
        .el()
        .set(space_between_items(), STREET)
        .with_padding_even(STREET),
        FlowColumn(vec![
            Button::new("Regular toggled", |_| {}).toggled(true).el(),
            Button::new("Primary toggled", |_| {})
                .toggled(true)
                .style(ButtonStyle::Primary)
                .el(),
            Button::new("Flat toggled", |_| {})
                .toggled(true)
                .style(ButtonStyle::Flat)
                .el(),
            Button::new(card_inner("Card toggled"), |_| {})
                .toggled(true)
                .style(ButtonStyle::Card)
                .el(),
            Button::new("Inline toggled", |_| {})
                .toggled(true)
                .style(ButtonStyle::Inline)
                .el(),
        ])
        .el()
        .set(space_between_items(), STREET)
        .with_padding_even(STREET),
        FlowColumn(vec![
            Button::new("Regular disabled", |_| {}).disabled(true).el(),
            Button::new("Primary disabled", |_| {})
                .disabled(true)
                .style(ButtonStyle::Primary)
                .el(),
            Button::new("Flat disabled", |_| {})
                .disabled(true)
                .style(ButtonStyle::Flat)
                .el(),
            Button::new(card_inner("Card disabled"), |_| {})
                .disabled(true)
                .style(ButtonStyle::Card)
                .el(),
            Button::new("Inline disabled", |_| {})
                .disabled(true)
                .style(ButtonStyle::Inline)
                .el(),
        ])
        .el()
        .set(space_between_items(), STREET)
        .with_padding_even(STREET),
        Button::new("\u{f1e2}", |_| {}).el(),
    ])
    .el()
    .set(space_between_items(), STREET)
    .set(translation(), vec3(100., 100., 0.))
}

#[main]
pub async fn main() -> EventResult {
    spawn_query((player(), user_id())).bind(move |players| {
        for (id, _) in players {
            entity::add_components(
                id,
                Entity::new()
                    .with_merge(make_orthographic_camera())
                    .with(orthographic_from_window(), EntityId::resources())
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
