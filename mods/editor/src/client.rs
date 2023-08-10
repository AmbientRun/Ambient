use ambient_api::{core::messages::Frame, input::CursorLockGuard, prelude::*};
use editor::{
    components::in_editor,
    messages::{Input, ToggleEditor},
};

#[main]
pub fn main() {
    let mut input_lock = None;
    let mut fixed_tick_last = game_time();

    let mut accumulated_aim_delta = Vec2::ZERO;

    Frame::subscribe(move |_| {
        let fixed_tick_dt = game_time() - fixed_tick_last;

        if !entity::get_component(player::get_local(), in_editor()).unwrap_or_default() {
            return;
        }

        let (delta, input) = input::get_delta();

        if delta.mouse_buttons.contains(&MouseButton::Right) {
            input_lock = Some(CursorLockGuard::new());
        } else if delta.mouse_buttons_released.contains(&MouseButton::Right) {
            input_lock = None;
        }

        let movement = [
            (KeyCode::W, -Vec2::Y),
            (KeyCode::S, Vec2::Y),
            (KeyCode::A, -Vec2::X),
            (KeyCode::D, Vec2::X),
        ]
        .iter()
        .filter(|(key, _)| input.keys.contains(key))
        .fold(Vec2::ZERO, |acc, (_, dir)| acc + *dir);

        if let Some(input_lock) = &mut input_lock {
            if input_lock.auto_unlock_on_escape(&input) {
                let speed = 4.0 * delta_time();
                accumulated_aim_delta += delta.mouse_position * speed;
            }
        }

        if fixed_tick_dt > Duration::from_millis(20) {
            if movement.length_squared() > 0.0 || accumulated_aim_delta.length_squared() > 0.0 {
                Input {
                    aim_delta: accumulated_aim_delta,
                    movement,
                    boost: input.keys.contains(&KeyCode::LShift),
                }
                .send_server_reliable();
            }

            accumulated_aim_delta = Vec2::ZERO;
            fixed_tick_last = game_time();
        }
    });

    App {}.el().spawn_interactive();
}

#[element_component]
pub fn App(hooks: &mut Hooks) -> Element {
    let in_editor = hooks
        .use_entity_component(player::get_local(), in_editor())
        .0
        .unwrap_or_default();

    hooks.use_keyboard_input(move |_, keycode, modifiers, pressed| {
        if modifiers == ModifiersState::empty() && keycode == Some(VirtualKeyCode::F5) && !pressed {
            ToggleEditor {}.send_server_reliable();
        }
    });

    // hack: I'm lazy
    hooks.use_spawn(move |_| {
        if !in_editor {
            ToggleEditor {}.send_server_reliable();
        }

        |_| {}
    });

    if in_editor {
        MenuBar::el()
    } else {
        Element::new()
    }
}

#[element_component]
fn MenuBar(hooks: &mut Hooks) -> Element {
    WindowSized::el([with_rect(
        FlowRow::el([Text::el(format!("Editor {}", env!("CARGO_PKG_VERSION")))])
            .with_padding_even(4.0),
    )
    .with(fit_horizontal(), Fit::Parent)
    .with_background(vec4(0.0, 0.0, 0.0, 0.5))])
}

// todo: move to API
pub fn fixed_rate_tick(dt: Duration, mut callback: impl FnMut(Duration) + 'static) {
    let mut last_tick = game_time();
    Frame::subscribe(move |_| {
        let delta = game_time() - last_tick;
        if delta < dt {
            return;
        }

        callback(delta);

        last_tick = game_time();
    });
}
