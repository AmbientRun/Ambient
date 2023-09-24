use ambient_api::{
    core::{
        physics::components::linear_velocity,
        rect::components::{background_color, line_from, line_to, line_width},
        transform::components::{local_to_world, rotation, translation},
    },
    element::use_entity_component,
    prelude::*,
};
use packages::{
    tangent_schema::{
        player::components as pc,
        vehicle::{client::components as vcc, components as vc},
    },
    this::messages::Input,
};

mod shared;

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

    spawn_query(translation())
        .requires(vc::is_vehicle())
        .bind(|vehicles| {
            for (_vehicle_id, translation) in vehicles {
                audio::SpatialAudioPlayer::oneshot(
                    translation,
                    packages::kenney_impact_sounds::assets::url("ImpactMining_003.ogg"),
                );
            }
        });

    CoreUI.el().spawn_interactive();
}

fn handle_input() {
    let mut last_input = input::get();
    let mut aim_direction = Vec2::ZERO;
    fixed_rate_tick(Duration::from_millis(20), move |_| {
        if !input::is_game_focused() {
            return;
        }

        let Some(local_vehicle) = entity::get_component(player::get_local(), pc::vehicle_ref())
        else {
            return;
        };

        let aim_direction_limits = entity::get_component(
            local_vehicle,
            packages::tangent_schema::vehicle::def::input::components::aim_direction_limits(),
        )
        .unwrap_or(Vec2::ONE * 20.0);

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

        aim_direction = (aim_direction + delta.mouse_position * 0.5)
            .clamp(-aim_direction_limits, aim_direction_limits);

        Input {
            direction,
            jump: input.keys.contains(&KeyCode::Space),
            fire: input.mouse_buttons.contains(&MouseButton::Left),
            aim_direction,
            respawn: delta.keys.contains(&KeyCode::K),
        }
        .send_server_unreliable();

        // Ensure we have a local copy of the aim direction that always reflects the most
        // recent state for the crosshair
        entity::add_component(
            player::get_local(),
            pc::input_aim_direction(),
            aim_direction,
        );

        last_input = input;
    });
}

#[element_component]
fn CoreUI(_hooks: &mut Hooks) -> Element {
    let vehicle_id = use_entity_component(_hooks, player::get_local(), pc::vehicle_ref());

    if let Some(vehicle_id) = vehicle_id {
        Crosshair::el(vehicle_id)
    } else {
        Element::new()
    }
}

#[element_component]
fn Crosshair(hooks: &mut Hooks, vehicle_id: EntityId) -> Element {
    let input_aim_direction =
        use_entity_component(hooks, player::get_local(), pc::input_aim_direction())
            .unwrap_or_default();

    let remote_aim_distance =
        use_entity_component(hooks, vehicle_id, vc::aim_distance()).unwrap_or(1_000.0);

    let Some(active_camera_id) = camera::get_active(None) else {
        return Element::new();
    };

    let aim_position =
        shared::calculate_aim_position(vehicle_id, input_aim_direction, remote_aim_distance);
    let pos_2d = camera::world_to_screen(active_camera_id, aim_position);

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
    .with(translation(), pos_2d.extend(0.1))
    .with(local_to_world(), default())
}
