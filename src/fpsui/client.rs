// TODO: add menu to choose game type
// TODO: add texteditor to type your name
// TODO: finish keybind to show/hide the scoreboard
// TODO: add a UI to show health

use ambient_api::{
    components::core::{
        app::window_logical_size,
        layout::{docking_bottom, space_between_items},
        player::player,
        rect::{background_color, line_from, line_to, line_width},
    },
    prelude::*,
};

#[main]
pub fn main() {
    App.el().spawn_interactive();
}

#[element_component]
pub fn App(hooks: &mut Hooks) -> Element {
    let (toggle, set_toggle) = hooks.use_state(false);
    let (ingame, set_ingame) = hooks.use_state(false);
    let (name, set_name) = hooks.use_state("".to_string());
    let players = hooks.use_query((
        player(),
        components::player_name(),
        components::player_killcount(),
        components::player_deathcount(),
    ));
    let (local_health, set_local_health) = hooks.use_state(100);

    hooks.use_frame(move |world| {
        let local_player = player::get_local();
        if let Ok(health) = world.get(local_player, components::player_health()) {
            set_local_health(health);
        }
    });

    let size_info = hooks.use_query(window_logical_size());

    let input = input::get();

    if input.keys.contains(&KeyCode::Tab) {
        set_toggle(true);
    } else {
        set_toggle(false);
    }

    let center_x = size_info[0].1.x as f32 / 2.;
    let center_y = size_info[0].1.y as f32 / 2.;

    if !ingame {
        FocusRoot::el([WindowSized(vec![
            FlowColumn::el([
                Text::el("enter your name blow. press enter to start the game."),
                TextEditor::new(name.clone(), set_name.clone())
                    .auto_focus()
                    .on_submit({
                        let set_ingame = set_ingame.clone();
                        move |v| {
                            set_ingame(true);
                            messages::StartGame::new(player::get_local(), v)
                                .send_server_unreliable();
                        }
                    })
                    .el(),
                Text::el("hint: use Tab to show/hide the scoreboard."),
            ])
            .with(space_between_items(), STREET), // .with_default(fit_horizontal_parent())
                                                  // .with_default(fit_vertical_parent())
        ])
        .el()
        // .with_default(align_horizontal_center())
        // .with_default(align_vertical_center())
        .with_padding_even(20.)])
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
                    WindowSized(vec![FlowColumn::el(
                        players
                            .iter()
                            .map(|(_id, (_, name, kill, death))| {
                                Text::el(format!(
                                    "\u{f007} {}    \u{f118} {}    \u{f119} {}",
                                    name, kill, death
                                ))
                            })
                            .collect::<Vec<_>>(),
                    )
                    .with(space_between_items(), STREET)])
                    .el()
                    .with_padding_even(20.)
                } else {
                    Element::new()
                }
            },
            WindowSized(vec![Dock::el([Text::el(format!(
                "health: {:?}",
                local_health
            ))
            // .header_style()
            .with_default(docking_bottom())
            .with_margin_even(10.)])])
            .el()
            .with_padding_even(20.),
        ])
    }
}
