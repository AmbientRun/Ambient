use std::sync::{Arc, Mutex};

use ambient_api::{
    core::{
        messages::Frame,
        physics::components::linear_velocity,
        rect::components::{background_color, line_from, line_to, line_width},
        transform::components::{local_to_world, rotation, translation},
    },
    prelude::*,
    ui::use_window_logical_resolution,
};
use packages::{
    game_object::player::components as gopc,
    tangent_schema::vehicle::{client::components as vcc, components as vc},
    this::messages::{Input, UseFailed},
};

#[main]
pub fn main() {
    query((rotation(), linear_velocity()))
        .requires(vc::is_vehicle())
        .each_frame(|vehicles| {
            for (id, (rot, lv)) in vehicles {
                entity::add_component(id, vcc::speed_kph(), lv.dot(rot * -Vec3::Y) * 3.6);
            }
        });

    handle_input();

    UseFailed::subscribe(|ctx, _msg| {
        if !ctx.server() {
            return;
        }

        let Some(translation) =
            entity::get_component(player::get_local(), gopc::control_of_entity())
                .and_then(|e| entity::get_component(e, translation()))
        else {
            return;
        };

        audio::SpatialAudioPlayer::oneshot(
            translation,
            packages::kenney_impact_sounds::assets::url("impactGlass_light_004.ogg"),
        );
    });

    CoreUI.el().spawn_interactive();
}

fn handle_input() {
    let mut last_input = input::get();
    // The most correct thing to do would be to store this in the ECS.
    let aim_direction = Arc::new(Mutex::new(Vec2::ZERO));

    Frame::subscribe({
        let aim_direction = aim_direction.clone();
        move |_| {
            let (delta, _) = input::get_delta();

            let mut aim_direction = aim_direction.lock().unwrap();
            *aim_direction += vec2(
                delta.mouse_position.x.to_radians(),
                delta.mouse_position.y.to_radians(),
            );
            aim_direction.y = aim_direction
                .y
                .clamp(-89f32.to_radians(), 89f32.to_radians());
        }
    });

    fixed_rate_tick(Duration::from_millis(20), move |_| {
        if !input::is_game_focused() {
            return;
        }

        let Some(camera_id) = camera::get_active(None) else {
            return;
        };

        let camera_ray = camera::clip_position_to_world_ray(camera_id, Vec2::ZERO);

        let input = input::get();
        let delta = input.delta(&last_input);
        let direction = {
            let mut direction = Vec2::ZERO;
            if input.keys.contains(&KeyCode::W) {
                direction.y += 1.;
            }
            if input.keys.contains(&KeyCode::S) {
                direction.y -= 1.;
            }
            if input.keys.contains(&KeyCode::A) {
                direction.x -= 1.;
            }
            if input.keys.contains(&KeyCode::D) {
                direction.x += 1.;
            }
            direction
        };

        Input {
            direction,
            jump: input.keys.contains(&KeyCode::Space),
            sprint: input.keys.contains(&KeyCode::LShift),
            use_button: input.keys.contains(&KeyCode::E),
            fire: input.mouse_buttons.contains(&MouseButton::Left),
            aim_direction: *aim_direction.lock().unwrap(),
            respawn: delta.keys.contains(&KeyCode::K),
            aim_ray_origin: camera_ray.origin,
            aim_ray_direction: camera_ray.dir,
        }
        .send_server_unreliable();

        last_input = input;
    });
}

#[element_component]
fn CoreUI(hooks: &mut Hooks) -> Element {
    let size = use_window_logical_resolution(hooks);
    Crosshair::el(size.as_vec2() / 2.0)
}

#[element_component]
fn Crosshair(_hooks: &mut Hooks, aim_position_2d: Vec2) -> Element {
    Group::el([
        Line.el()
            .with(line_from(), vec3(-10.0, -10.0, 0.))
            .with(line_to(), vec3(10.0, 10.0, 0.))
            .with(line_width(), 1.)
            .with(background_color(), vec4(1., 1., 1., 1.)),
        Line.el()
            .with(line_from(), vec3(-10.0, 10.0, 0.))
            .with(line_to(), vec3(10.0, -10.0, 0.))
            .with(line_width(), 1.)
            .with(background_color(), vec4(1., 1., 1., 1.)),
    ])
    .with(translation(), aim_position_2d.extend(0.1))
    .with(local_to_world(), default())
}
