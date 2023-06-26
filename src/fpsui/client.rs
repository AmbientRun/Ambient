// TODO: add menu to choose game type
// TODO: add texteditor to type your name
// TODO: finish keybind to show/hide the scoreboard
// TODO: add a UI to show health

use ambient_api::prelude::*;

#[main]
pub fn main() {
    App.el().spawn_interactive();
}

#[element_component]
pub fn App(hooks: &mut Hooks) -> Element {
    let (toggle, set_toggle) = hooks.use_state(false);
    let (ingame, set_ingame) = hooks.use_state(false);
    let (name, set_name) = hooks.use_state("".to_string());
    let players = hooks.use_query((player(), components::player_health()));
    let size_info = hooks.use_query(window_logical_size());

    let input = input::get();

    if input.keys.contains(&KeyCode::Tab) {
        set_toggle(true);
    } else {
        set_toggle(false);
    }

    // for (resource_id, xy) in &size_info {
    //     println!("window size change: {:?} {:?}", resource_id, xy);
    // }

    let center_x = size_info[0].1.x as f32 / 2.;
    let center_y = size_info[0].1.y as f32 / 2.;

    if !ingame {
        FocusRoot::el([FlowColumn::el([
            Text::el("Enter your name:"),
            TextEditor::new(name.clone(), set_name.clone())
                .auto_focus()
                .on_submit({
                    let set_ingame = set_ingame.clone();
                    let name = name.clone();
                    move |_| {
                        set_ingame(true);
                        messages::StartGame {
                            name: name.clone(),
                            id: player::get_local(),
                            uid: player::get_local().to_string(),
                            // uid: user_id::get_local(),
                        }
                        .send_server_unreliable();
                    }
                })
                .el(),
            Button::new("Join game", move |_| {
                set_ingame(true);
                messages::StartGame {
                    name: name.clone(),
                    id: player::get_local(),
                    uid: player::get_local().to_string(),
                }
                .send_server_unreliable();
            })
            .el(),
        ])])
    } else {
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
            {
                if toggle {
                    FlowColumn::el(
                        players
                            .iter()
                            .map(|(id, (_, health))| {
                                Text::el(format!("id: {:?}, health: {}", id, health))
                            })
                            .collect::<Vec<_>>(),
                    )
                    .with(margin(), vec4(10., 10., 10., 10.))
                } else {
                    Element::new()
                }
            },
        ])
    }
}
