use ambient_api::{
    components::core::{layout::space_between_items, rendering::color, text::font_size},
    prelude::*,
};

#[element_component]
fn App(_hooks: &mut Hooks) -> Element {
    FlowColumn::el([
        Text::el("Header").header_style(),
        Text::el("Section").section_style(),
        Text::el("Default text \u{f1e2} \u{fb8f}"),
        Text::el("Small").small_style(),
        Separator { vertical: false }.el(),
        Text::el("Custom size").with(font_size(), 40.),
        Text::el("Custom color").with(color(), vec4(1., 0., 0., 1.)),
        Text::el("Multi\n\nLine"),
    ])
    .with_padding_even(STREET)
    .with(space_between_items(), 10.)
}

#[main]
pub fn main() {
    App.el().spawn_interactive();
}
