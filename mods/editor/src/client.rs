use ambient_api::{
    core::{
        app::components::name, messages::Frame, rendering::components::color,
        transform::components::translation,
    },
    input::CursorLockGuard,
    prelude::*,
};
use editor::{
    components::{editor_camera, in_editor, mouseover_entity, mouseover_position},
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

        let Some(camera_id) = entity::get_component(player::get_local(), editor_camera()) else { return; };

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
            let ray = camera::screen_position_to_world_ray(camera_id, input.mouse_position);

            Input {
                aim_delta: accumulated_aim_delta,
                movement,
                boost: input.keys.contains(&KeyCode::LShift),
                ray_origin: ray.origin,
                ray_direction: ray.dir,
            }
            .send_server_reliable();

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
        Group::el([MenuBar::el(), MouseoverDisplay::el()])
    } else {
        Element::new()
    }
}

#[element_component]
fn MenuBar(_hooks: &mut Hooks) -> Element {
    WindowSized::el([with_rect(
        FlowRow::el([Text::el(format!("Editor {}", env!("CARGO_PKG_VERSION")))])
            .with_padding_even(4.0),
    )
    .with(fit_horizontal(), Fit::Parent)
    .with_background(vec4(0.0, 0.0, 0.0, 0.5))])
}

#[element_component]
fn MouseoverDisplay(hooks: &mut Hooks) -> Element {
    let player_id = player::get_local();
    let (mouseover_position, _) = hooks.use_entity_component(player_id, mouseover_position());
    let (mouseover_entity, _) = hooks.use_entity_component(player_id, mouseover_entity());
    let (camera_id, _) = hooks.use_entity_component(player_id, editor_camera());

    let Some(mouseover_position) = mouseover_position else { return Element::new(); };
    let Some(camera_id) = camera_id else { return Element::new(); };

    let mut text = format!("{:.02?}", mouseover_position.to_array());
    if let Some(mouseover_entity) = mouseover_entity {
        text += &format!(
            "\n{}",
            entity::get_component(mouseover_entity, name())
                .unwrap_or_else(|| mouseover_entity.to_string())
        );
    }

    let mouseover_position_2d = camera::world_to_screen(camera_id, mouseover_position).extend(0.0);
    Text::el(text)
        .with(translation(), mouseover_position_2d)
        .with(color(), Vec4::ONE)
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
