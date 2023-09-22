use ambient_api::{
    core::{
        primitives::concepts::Sphere, rendering::components::color, transform::components::scale,
    },
    prelude::*,
};

use packages::this::{components::*, concepts::Explosion};

#[main]
pub fn main() {
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
                .with(created_at(), game_time())
                .with(scale(), start.0 * Vec3::ONE)
                .with(color(), start.1.extend(1.0)),
            );
        }
    });

    query((created_at(), radius()))
        .requires(is_explosion())
        .each_frame(|explosions| {
            for (id, (created_at, radius)) in explosions {
                let time = (game_time() - created_at).as_secs_f32();
                let (scale, color) = explosion_scale_and_color(time, radius);
                entity::set_component(id, self::scale(), scale * Vec3::ONE);
                entity::set_component(id, self::color(), color.extend(1.0));
            }
        });
}

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
