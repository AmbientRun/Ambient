// TODO: add menu to choose game type

use ambient_api::{
    components::core::{
        app::window_logical_size,
        layout::{docking_bottom, min_width, space_between_items},
        player::player,
        rect::{background_color, line_from, line_to, line_width},
    },
    prelude::*,
    ui::HooksExt,
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

    FocusRoot::el([WindowSized::el([FlowColumn::el([
        Text::el("enter your name below. press enter to start the game."),
        TextEditor::new(name.clone(), set_name.clone())
            .auto_focus()
            .on_submit(|v| messages::StartGame::new(v).send_server_reliable())
            .el()
            .with(min_width(), 100.0),
        Text::el("hint: hold Tab to toggle the scoreboard."),
    ])
    .with(space_between_items(), STREET)])
    .with_padding_even(20.)])
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
    let (size, _) = hooks.use_resource(window_logical_size());
    let size = size.unwrap_or_default();
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
