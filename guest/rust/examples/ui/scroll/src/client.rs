use ambient_api::prelude::*;

#[element_component]
fn App(hooks: &mut Hooks) -> Element {
    let ratio = hooks.use_query(window_scale_factor());
    FlowColumn::el([
        Text::el("ScrollBoxView")
        .header_style(),
        ScrollBoxView {
            min_width: 150.0,
            min_height: 150.0,
            scroll_height: 100.0,
            inner: FlowColumn::el([
                Text::el("1111111111111111")
                .header_style(),
                Text::el("2222222222222222")
                .header_style(),
                Text::el("3333333333333333")
                .header_style(),
                Text::el("4444444444444444")
                .header_style(),
                Text::el("5555555555555555")
                .header_style(),
                Text::el("6666666666666666")
                .header_style(),
                Text::el("7777777777777777")
                .header_style(),
                Text::el("8888888888888888")
                .header_style(),
                Text::el("9999999999999999")
                .header_style(),
            ])
        }.el()
    ])
}

#[main]
pub fn main() {
    App.el().spawn_interactive();
}
