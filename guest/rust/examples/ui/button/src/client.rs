use ambient_api::{components::core::layout::space_between_items, prelude::*};

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
            Button::new("Regular", |_| println!("Regular pressed"))
                .hotkey(VirtualKeyCode::Space)
                .el(),
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
        .with(space_between_items(), STREET)
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
        .with(space_between_items(), STREET)
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
        .with(space_between_items(), STREET)
        .with_padding_even(STREET),
        Button::new("\u{f1e2}", |_| {}).el(),
    ])
    .el()
    .with(space_between_items(), STREET)
    .with_padding_even(STREET)
}

#[main]
pub fn main() {
    App.el().spawn_interactive();
}
