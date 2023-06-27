use ambient_api::{
    components::core::layout::{height, space_between_items, width},
    prelude::*,
};

#[element_component]
fn App(_hooks: &mut Hooks) -> Element {
    WindowSized::el(vec![ScrollArea::el(
        ScrollAreaSizing::FitParentWidth,
        FlowColumn::el([
            Text::el("1 ScrollAreaSizing::FitParentWidth").header_style(),
            Text::el("2 ScrollAreaSizing::FitParentWidth").header_style(),
            Text::el("3 ScrollAreaSizing::FitParentWidth").header_style(),
            ScrollArea::el(
                ScrollAreaSizing::FitChildrenWidth,
                FlowColumn::el([
                    Text::el("1 ScrollAreaSizing::FitChildrenWidth"),
                    Text::el("2 ScrollAreaSizing::FitChildrenWidth"),
                    Text::el("3 ScrollAreaSizing::FitChildrenWidth"),
                    Text::el("4 ScrollAreaSizing::FitChildrenWidth"),
                    Text::el("5 ScrollAreaSizing::FitChildrenWidth"),
                    Text::el("6 ScrollAreaSizing::FitChildrenWidth"),
                    Text::el("7 ScrollAreaSizing::FitChildrenWidth"),
                    Text::el("8 ScrollAreaSizing::FitChildrenWidth"),
                    Text::el("9 ScrollAreaSizing::FitChildrenWidth"),
                    Text::el("10 ScrollAreaSizing::FitChildrenWidth"),
                    Text::el("11 ScrollAreaSizing::FitChildrenWidth"),
                    Text::el("12 ScrollAreaSizing::FitChildrenWidth"),
                    Text::el("12 ScrollAreaSizing::FitChildrenWidth"),
                    Text::el("13 ScrollAreaSizing::FitChildrenWidth"),
                    Text::el("14 ScrollAreaSizing::FitChildrenWidth"),
                    Text::el("15 ScrollAreaSizing::FitChildrenWidth"),
                ]),
            )
            .with(height(), 100.) // specified height
            .with(width(), 300.), // specified width,
            Text::el("4 ScrollAreaSizing::FitParentWidth").header_style(),
            Text::el("5 ScrollAreaSizing::FitParentWidth").header_style(),
            Text::el("6 ScrollAreaSizing::FitParentWidth").header_style(),
            Text::el("7 ScrollAreaSizing::FitParentWidth").header_style(),
            Text::el("8 ScrollAreaSizing::FitParentWidth").header_style(),
            Text::el("9 ScrollAreaSizing::FitParentWidth").header_style(),
            Text::el("10 ScrollAreaSizing::FitParentWidth").header_style(),
            Text::el("11 ScrollAreaSizing::FitParentWidth").header_style(),
            Text::el("12 ScrollAreaSizing::FitParentWidth").header_style(),
            Text::el("13 ScrollAreaSizing::FitParentWidth").header_style(),
            Text::el("14 ScrollAreaSizing::FitParentWidth").header_style(),
            Text::el("15 ScrollAreaSizing::FitParentWidth").header_style(),
            Text::el("16 ScrollAreaSizing::FitParentWidth").header_style(),
            Text::el("17 ScrollAreaSizing::FitParentWidth").header_style(),
            Text::el("18 ScrollAreaSizing::FitParentWidth").header_style(),
            Text::el("19 ScrollAreaSizing::FitParentWidth").header_style(),
            Text::el("20 ScrollAreaSizing::FitParentWidth").header_style(),
        ])
        .with(space_between_items(), STREET),
    )])
}

#[main]
pub fn main() {
    App.el().spawn_interactive();
}
