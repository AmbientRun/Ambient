// TODO: add menu to choose game type

use ambient_api::{
    components::core::{
        layout::{docking_bottom, height, min_height, min_width, space_between_items, width},
        player::player,
        rect::{background_color, line_from, line_to, line_width},
    },
    prelude::*,
    ui::ImageFromUrl,
};

#[main]
pub fn main() {
    App.el().spawn_interactive();
}

#[element_component]
pub fn App(hooks: &mut Hooks) -> Element {
    let (player_name, _) =
        hooks.use_entity_component(player::get_local(), components::player_name());

    if player_name.is_none() {
        JoinScreen::el()
    } else {
        GameUI::el()
    }
}

#[element_component]
fn JoinScreen(hooks: &mut Hooks) -> Element {
    use_input_request(hooks);
    let (name, set_name) = hooks.use_state("".to_string());

    FocusRoot::el([
        WindowSized::el([FlowColumn::el([
            Text::el("APFS").header_style(),
            Separator { vertical: false }.el(),
            Text::el("enter your name below. press enter to start the game."),
            TextEditor::new(name.clone(), set_name.clone())
                .auto_focus()
                .on_submit(|v| messages::StartGame::new(v).send_server_reliable())
                .el()
                .with(background_color(), vec4(0.3, 0.3, 0.3, 0.6))
                .with(min_height(), 16.0)
                .with(min_width(), 100.0),
            Separator { vertical: false }.el(),
            Text::el("rules:").section_style(),
            Text::el("shoot every one you see to death to gain a point."),
            Text::el("once you die, you lose a point and will be respawned."),
            Separator { vertical: false }.el(),
            Text::el("control:").section_style(),
            Text::el("use your mouse to look around; shoot with left click."),
            Text::el("right click to zoom."),
            Text::el("hold [WASD] to move."),
            Text::el("tap [Space] to jump."),
            Text::el("hold [Shift] to run."),
            Text::el("hold [Tab] to toggle the scoreboard."),
            Text::el("modding:").section_style(),
            Text::el("use [F1] to toggle the console."),
            Text::el("use [F2] to open the Ember Loader."),
            Text::el("use [F3] to open the WASM Manager."),
        ])
        .with(space_between_items(), STREET)])
        .with_padding_even(20.),
        ImageFromUrl {
            url: asset::url("assets/afps.png").unwrap(),
        }
        .el()
        .with(width(), hooks.use_window_logical_resolution().x as f32)
        .with(height(), hooks.use_window_logical_resolution().y as f32),
    ])
}

#[element_component]
fn GameUI(hooks: &mut Hooks) -> Element {
    let (scoreboard_open, set_scoreboard_open) = hooks.use_state(false);
    hooks.use_keyboard_input(move |_, keycode, _, pressed| {
        if keycode == Some(VirtualKeyCode::Tab) {
            set_scoreboard_open(pressed);
        }
    });

    Group::el([
        Crosshair::el(),
        Hud::el(),
        if scoreboard_open {
            Scoreboard::el()
        } else {
            Element::new()
        },
    ])
}

// TODO: there is *definitely* a better way to put the crosshair in the centre of the screen
#[element_component]
fn Crosshair(hooks: &mut Hooks) -> Element {
    let size = hooks.use_window_logical_resolution();
    let center_x = size.x as f32 / 2.;
    let center_y = size.y as f32 / 2.;

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
    ])
}

#[element_component]
fn Hud(hooks: &mut Hooks) -> Element {
    let (local_health, set_local_health) = hooks.use_state(100);
    hooks.use_frame(move |world| {
        let local_player = player::get_local();
        if let Ok(health) = world.get(local_player, components::player_health()) {
            set_local_health(health);
        }
    });

    WindowSized::el([Dock::el([Text::el(format!("health: {:?}", local_health))
        // .header_style()
        .with_default(docking_bottom())
        .with_margin_even(10.)])])
    .with_padding_even(20.)
}

#[element_component]
fn Scoreboard(hooks: &mut Hooks) -> Element {
    use_input_request(hooks);

    let players = hooks.use_query((
        player(),
        components::player_name(),
        components::player_killcount(),
        components::player_deathcount(),
    ));

    WindowSized::el([FlowColumn::el(
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
    .with_padding_even(20.)
}

/// Requests input from the user, and releases it when the element is unmounted.
fn use_input_request(hooks: &mut Hooks<'_>) {
    hooks.use_spawn(|_| {
        messages::RequestInput {}.send_local_broadcast(false);
        |_| {
            messages::ReleaseInput {}.send_local_broadcast(false);
        }
    })
}
