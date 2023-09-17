use ambient_api::{
    core::{
        app::components::name,
        physics::components::linear_velocity,
        primitives::concepts::Sphere,
        rendering::components::color,
        transform::components::{rotation, scale},
    },
    once_cell::sync::Lazy,
    prelude::*,
};
use packages::{
    tangent_schema::{
        concepts::Explosion,
        explosion::components as ec,
        vehicle::{client::components as vcc, components as vc},
    },
    this::messages::{Input, OnCollision, OnSpawn},
};

#[main]
pub fn main() {
    // HACK: despawn all wheels on spawn
    spawn_query(name()).bind(|entities| {
        for (id, name) in entities {
            if name.starts_with("wheel") {
                entity::despawn(id);
            }
        }
    });

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

    OnSpawn::subscribe(|ctx, msg| {
        if !ctx.server() {
            return;
        }

        audio::SpatialAudioPlayer::oneshot(
            msg.position,
            packages::kenney_impact_sounds::assets::url("ImpactMining_003.ogg"),
        );
    });
}

fn handle_input() {
    let mut last_input = input::get();
    fixed_rate_tick(Duration::from_millis(20), move |_| {
        if !input::is_game_focused() {
            return;
        }

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
            suicide: delta.keys.contains(&KeyCode::K),
        }
        .send_server_unreliable();

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
