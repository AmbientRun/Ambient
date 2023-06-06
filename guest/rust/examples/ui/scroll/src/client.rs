use ambient_api::prelude::*;

#[element_component]
fn App(hooks: &mut Hooks) -> Element {
    let size_info = hooks.use_query(window_logical_size());
    let x = size_info[0].1.x as f32;
    let y = size_info[0].1.y as f32;
    let f = FlowColumn::el([
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
        ]),
        Text::el("Shall I compare thee to a summer's day?")
        .header_style(),
        Text::el("Thou art more lovely and more temperate:")
        .header_style(),
        Text::el("Rough winds do shake the darling buds of May,")
        .header_style(),
        Text::el("And summer's lease hath all too short a date:")
        .header_style(),
        Text::el("Sometime too hot the eye of heaven shines,")
        .header_style(),
        Text::el("And often is his gold complexion dimm'd;")
        .header_style(),
        Text::el("And every fair from fair sometime declines,")
        .header_style(),
        Text::el("By chance, or nature's changing course, untrimm'd;")
        .header_style(),
        Text::el("But thy eternal summer shall not fade,")
        .header_style(),
        Text::el("Nor lose possession of that fair thou ow'st;")
        .header_style(),
        Text::el("Nor shall Death brag thou wander'st in his shade,")
        .header_style(),
        Text::el("When in eternal lines to time thou grow'st;")
        .header_style(),

    ])
    .with_padding_even(STREET)
    .with(space_between_items(), 10.);

    ScrollBoxView {
        min_width: x,
        min_height: y,
        scroll_height: y/2.0,
        inner: f
    }.el()
}

#[main]
pub fn main() {
    App.el().spawn_interactive();
}
