use ambient_api::prelude::*;

#[element_component]
fn App(_hooks: &mut Hooks) -> Element {

    let e = FlowColumn::el([
        Text::el("Header").header_style(),
        Text::el("Section").section_style(),
        Text::el("Default text \u{f1e2} \u{fb8f}"),
        Text::el("Small").small_style(),
        Separator { vertical: false }.el(),
        Text::el("Custom size").with(font_size(), 40.),
        Text::el("Custom color").with(color(), vec4(1., 0., 0., 1.)),
        Text::el("Multi\n\n\nLine"),
        Text::el("Now").header_style(),
        Text::el("we").header_style(),
        Text::el("add").header_style(),
        Text::el("some").header_style(),
        Text::el("headers").header_style(),
        Text::el("to").header_style(),
        Text::el("test").header_style(),
        Text::el("the").header_style(),
        Text::el("scroll").header_style(),
        Text::el("bar").header_style(),
    ])
    .with_padding_even(STREET)
    .with(space_between_items(), 10.);

    ScrollArea::el(
        ScrollAreaSizing::MaxScrollDown(100.0),
        e
    )
}

#[main]
pub fn main() {
    App.el().spawn_interactive();
}
