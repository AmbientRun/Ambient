use ambient_api::{
    core::{
        physics::components::linear_velocity,
        primitives::concepts::Sphere,
        rect::components::{background_color, line_from, line_to, line_width},
        rendering::components::color,
        transform::components::{local_to_world, rotation, scale, translation},
    },
    element::use_entity_component,
    once_cell::sync::Lazy,
    prelude::*,
};
use packages::{
    tangent_schema::{
        concepts::Explosion,
        explosion::components as ec,
        player::components as pc,
        vehicle::{client::components as vcc, components as vc},
    },
    this::messages::{Input, OnCollision},
};

mod shared;

#[main]
pub fn main() {
    query((rotation(), linear_velocity()))
        .requires(vc::player_ref())
        .each_frame(|vehicles| {
            for (id, (rot, lv)) in vehicles {
                entity::add_component(id, vcc::speed_kph(), lv.dot(rot * -Vec3::Y) * 3.6);
            }
        });

    handle_input();
    handle_collisions();
    handle_explosions();

    spawn_query(translation())
        .requires(vc::player_ref())
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
            packages::tangent_schema::vehicle::data::input::components::aim_direction_limits(),
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

fn handle_collisions() {
    static SOUNDS: Lazy<[Vec<String>; 3]> = Lazy::new(|| {
        let url = |ty, idx| {
            packages::kenney_impact_sounds::assets::url(&format!("impactPlate_{ty}_{idx:0>3}.ogg"))
        };

        ["light", "medium", "heavy"].map(|ty| (0..5).map(|idx| url(ty, idx)).collect())
    });

    OnCollision::subscribe(|ctx, msg| {
        if !ctx.server() {
            return;
        }

        let impact_type = match msg.speed {
            speed if speed < 5. => 0,
            speed if speed < 10. => 1,
            _ => 2,
        };

        let sound = SOUNDS[impact_type].choose(&mut thread_rng()).unwrap();
        audio::SpatialAudioPlayer::oneshot(msg.position, sound);
    });
}

fn handle_explosions() {
    fn explosion_scale_and_color(time: f32, radius: f32) -> (f32, Vec3) {
        struct Point {
            time: f32,
            size: f32,
            color: Vec3,
        }

        let points = [
            Point {
                time: 0.0,
                size: 0.001,
                color: vec3(1.00, 0.29, 0.24),
            },
            Point {
                time: 0.1,
                size: radius,
                color: vec3(1.00, 0.85, 0.24),
            },
            Point {
                time: 0.5,
                size: 0.001,
                color: vec3(0.57, 0.00, 0.17),
            },
        ];

        for arr in points.windows(2) {
            let [a, b] = arr else {
                break;
            };
            if time >= a.time && time < b.time {
                let t = (time - a.time) / (b.time - a.time);
                return (lerp(a.size, b.size, t), a.color.lerp(b.color, t));
            }
        }

        points.last().map(|p| (p.size, p.color)).unwrap()
    }

    spawn_query(Explosion::as_query()).bind(|explosions| {
        for (id, explosion) in explosions {
            audio::SpatialAudioPlayer::oneshot(
                explosion.translation,
                packages::this::assets::url("audio/587194__derplayer__explosion_big_01.ogg"),
            );

            let start = explosion_scale_and_color(0.0, explosion.radius);

            entity::add_components(
                id,
                Sphere {
                    sphere_radius: 1.0,
                    ..Sphere::suggested()
                }
                .make()
                .with(ec::created_at(), game_time())
                .with(scale(), start.0 * Vec3::ONE)
                .with(color(), start.1.extend(1.0)),
            );
        }
    });

    query((ec::created_at(), ec::radius()))
        .requires(ec::is_explosion())
        .each_frame(|explosions| {
            for (id, (created_at, radius)) in explosions {
                let time = (game_time() - created_at).as_secs_f32();
                let (scale, color) = explosion_scale_and_color(time, radius);
                entity::set_component(id, self::scale(), scale * Vec3::ONE);
                entity::set_component(id, self::color(), color.extend(1.0));
            }
        });
}

#[element_component]
fn CoreUI(_hooks: &mut Hooks) -> Element {
    let vehicle_id = use_entity_component(_hooks, player::get_local(), pc::vehicle_ref()).0;

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
            .0
            .unwrap_or_default();

    let Some(active_camera_id) = camera::get_active(None) else {
        return Element::new();
    };

    let aim_position = shared::calculate_aim_position(vehicle_id, input_aim_direction);
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
