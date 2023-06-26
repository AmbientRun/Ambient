// TODO: add menu to choose game type
// TODO: add texteditor to type your name
// TODO: add a keybind to show/hide the scoreboard

use ambient_api::prelude::*;

#[main]
pub fn main() {
    App.el().spawn_interactive();
}

#[element_component]
pub fn App(hooks: &mut Hooks) -> Element {
    let (toggle, set_toggle) = hooks.use_state(false);
    let players = hooks.use_query(player());
    let size_info = hooks.use_query(window_logical_size());
    for (resource_id, xy) in &size_info {
        println!("window size change: {:?} {:?}", resource_id, xy);
    }
    let center_x = size_info[0].1.x as f32 / 2.;
    let center_y = size_info[0].1.y as f32 / 2.;

    Group::el([
        Line.el()
            .with(line_from(), vec3(center_x - 10., center_y, 0.))
            .with(line_to(), vec3(center_x + 10., center_y, 0.))
            .with(line_width(), 2.)
            .with(background_color(), vec4(1., 1., 1., 1.)),
        Line.el()
            .with(line_from(), vec3(center_x, center_y - 10., 0.))
            .with(line_to(), vec3(center_x, center_y + 10., 0.))
            .with(line_width(), 2.)
            .with(background_color(), vec4(1., 1., 1., 1.)),
        Hotkey::new(
            VirtualKeyCode::Tab,
            {
                let set_toggle = set_toggle.clone();
                move |_w| {
                    set_toggle(!toggle);
                }
            },
            {
                if toggle {
                    FlowColumn::el(
                        players
                            .iter()
                            .map(|x| Text::el(format!("{:?}", x.0)))
                            .collect::<Vec<_>>(),
                    )
                } else {
                    Element::new()
                }
            },
        )
        .into(),
    ])
}
