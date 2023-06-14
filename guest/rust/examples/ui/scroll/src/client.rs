use ambient_api::prelude::*;

#[element_component]
fn App(_hooks: &mut Hooks) -> Element {
    WindowSized::el(vec![ScrollArea::el(
        ScrollAreaSizing::FitParentWidth,
        FlowColumn::el([
            Text::el("1 Test Test Test").header_style(),
            Text::el("2 Test Test Test").header_style(),
            Text::el("3 Test Test Test").header_style(),
            UIBase::el()
                .with(height(), 100.) // specified height
                .with(width(), 300.) // specified width
                // .with(background_color(), vec4(0., 0., 0.3, 0.6))
                .children(vec![ScrollArea::el(
                    ScrollAreaSizing::FitChildrenWidth,
                    FlowColumn::el([
                        Text::el("ScrollAreaSizing::FitChildrenWidth"),
                        Text::el("ScrollAreaSizing::FitChildrenWidth"),
                        Text::el("ScrollAreaSizing::FitChildrenWidth"),
                        Text::el("ScrollAreaSizing::FitChildrenWidth"),
                        Text::el("ScrollAreaSizing::FitChildrenWidth"),
                        Text::el("ScrollAreaSizing::FitChildrenWidth"),
                        Text::el("ScrollAreaSizing::FitChildrenWidth"),
                        Text::el("ScrollAreaSizing::FitChildrenWidth"),
                        Text::el("ScrollAreaSizing::FitChildrenWidth"),
                        Text::el("ScrollAreaSizing::FitChildrenWidth"),
                        Text::el("ScrollAreaSizing::FitChildrenWidth"),
                        Text::el("ScrollAreaSizing::FitChildrenWidth"),
                        Text::el("ScrollAreaSizing::FitChildrenWidth"),
                        Text::el("ScrollAreaSizing::FitChildrenWidth"),
                        Text::el("ScrollAreaSizing::FitChildrenWidth"),
                        Text::el("ScrollAreaSizing::FitChildrenWidth"),
                    ]),
                )]),
            Text::el("4 Test Test Test").header_style(),
            Text::el("5 Test Test Test").header_style(),
            Text::el("6 Test Test Test").header_style(),
            Text::el("7 Test Test Test").header_style(),
            Text::el("8 Test Test Test").header_style(),
            Text::el("9 Test Test Test").header_style(),
            Text::el("10 Test Test Test").header_style(),
            Text::el("11 Test Test Test").header_style(),
            Text::el("12 Test Test Test").header_style(),
            Text::el("13 Test Test Test").header_style(),
            Text::el("14 Test Test Test").header_style(),
            Text::el("15 Test Test Test").header_style(),
            Text::el("16 Test Test Test").header_style(),
            Text::el("17 Test Test Test").header_style(),
            Text::el("18 Test Test Test").header_style(),
            Text::el("19 Test Test Test").header_style(),
            Text::el("20 Test Test Test").header_style(),
        ])
        .with(space_between_items(), STREET),
    )])
}

#[main]
pub fn main() {
    App.el().spawn_interactive();
}
