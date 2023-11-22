use ambient_api::{
    core::{
        app::components::name,
        camera::concepts::{
            PerspectiveInfiniteReverseCamera, PerspectiveInfiniteReverseCameraOptional,
        },
        primitives::components::{cube, quad},
        transform::components::{lookat_target, translation},
    },
    prelude::*,
};

use packages::tuners::{components::output, concepts::Tuner};
use packages::{this::components::*, tuners::components::tuner_min};

#[main]
pub fn main() {
    PerspectiveInfiniteReverseCamera {
        optional: PerspectiveInfiniteReverseCameraOptional {
            aspect_ratio_from_window: Some(entity::resources()),
            main_scene: Some(()),
            translation: Some(Vec3::ONE * 25.),
            ..default()
        },
        ..PerspectiveInfiniteReverseCamera::suggested()
    }
    .make()
    .with(lookat_target(), vec3(0., 0., 0.))
    .spawn();

    for _ in 0..100 {
        spawn_boid();
    }

    init_boids_logic();

    println!("Hello, Ambient!");
}

fn spawn_boid() {
    Entity::new()
        .with(
            translation(),
            (random::<Vec2>() - 0.5).extend(0.) * 25. * 2.,
        )
        .with(cube(), ())
        .with(is_boid(), ())
        .with(
            boid_velocity(),
            (random::<Vec2>() - 0.5).extend(0.) * 50. * 2.,
        )
        .spawn();
}

fn init_boids_logic() {
    // boids quantity
    {
        let quantity_tuner = mk_tuner("Num of Boids", 51, 1001);
        entity::set_component(quantity_tuner, tuner_min(), 1.); // minimum 1

        query(()).requires(is_boid()).each_frame(move |boids| {
            let target_quantity: usize = entity::get_component(quantity_tuner, output())
                .unwrap_or(1.)
                .ceil() as usize;
            let to_target_quantity: i32 =
                (target_quantity as i32 - boids.len() as i32).max(-(boids.len() as i32 - 1));
            if to_target_quantity > 0 {
                for _ in 0..to_target_quantity {
                    spawn_boid();
                }
            }
            if to_target_quantity < 0 {
                let mut i = 0;
                for (boid, _) in boids {
                    entity::despawn(boid);
                    i -= 1;
                    if i <= to_target_quantity {
                        break;
                    }
                }
            }
        });
    }

    // basic velocity and speed limit
    {
        query((translation(), boid_velocity()))
            .requires(is_boid())
            .each_frame(|boids| {
                let dt = delta_time();
                let maxspeed = 50.;
                for (boid, (pos, mut vel)) in boids {
                    if vel.length_squared() > maxspeed * maxspeed {
                        vel = vel.normalize() * maxspeed;
                        entity::set_component(boid, boid_velocity(), vel);
                    }
                    entity::set_component(boid, translation(), pos + vel * dt);
                }
            });
    }

    let match_dist_tuner = mk_tuner("Match Range", 7, 20);

    // to center
    {
        let posmatch_dist_tuner = match_dist_tuner.clone();
        let posmatch_str_tuner = mk_tuner("Match Position (Coherence)", 5, 25);

        query(translation())
            .requires(is_boid())
            .each_frame(move |boids| {
                let dt = delta_time();
                let posmatch_dist: f32 =
                    entity::get_component(posmatch_dist_tuner, output()).unwrap_or(1.);
                // println!("posmatch_dist = {posmatch_dist}");
                let posmatch_strength: f32 =
                    entity::get_component(posmatch_str_tuner, output()).unwrap_or(1.);
                for (boid, pos) in &boids {
                    let mut perceived_center = Vec3::ZERO;
                    let mut bn = 0;
                    for (oboid, opos) in &boids {
                        if oboid != boid
                            && opos.distance_squared(*pos) < posmatch_dist * posmatch_dist
                        {
                            perceived_center += *opos;
                            bn += 1;
                        }
                    }

                    if bn > 0 {
                        perceived_center /= bn as f32;

                        entity::mutate_component(*boid, boid_velocity(), move |v| {
                            *v += (perceived_center - *pos) * posmatch_strength * dt;
                        });
                    }
                }
            });
    }

    // velocity matching
    {
        let velmatch_dist_tuner = match_dist_tuner.clone();
        let velmatch_str_tuner = mk_tuner("Match Velocity (Alignment)", 5, 25);
        query((translation(), boid_velocity()))
            .requires(is_boid())
            .each_frame(move |boids| {
                let dt = delta_time();
                let velmatch_dist: f32 =
                    entity::get_component(velmatch_dist_tuner, output()).unwrap_or(1.);
                let velmatch_strength: f32 =
                    entity::get_component(velmatch_str_tuner, output()).unwrap_or(1.);
                for (boid, (pos, _)) in &boids {
                    let mut total_velocity = Vec3::ZERO;
                    let mut bns = 0;
                    for (oboid, (opos, ovel)) in &boids {
                        if oboid != boid
                            && opos.distance_squared(*pos) < velmatch_dist * velmatch_dist
                        {
                            total_velocity += *ovel;
                            bns += 0;
                        }
                    }
                    if bns > 0 {
                        // total_velocity /= bns as f32;
                        entity::mutate_component(*boid, boid_velocity(), move |v| {
                            *v += (total_velocity / bns as f32 - *v) * velmatch_strength * dt;
                        });
                    }
                }
            });
    }

    // repulsion
    {
        let repulsive_dist_tuner = mk_tuner("Avoid Dist", 2, 10);
        let repulsive_str_tuner = mk_tuner("Avoid Str", 3, 10);

        query(translation())
            .requires(is_boid())
            .each_frame(move |boids| {
                let dt = delta_time();
                let repulsive_dist: f32 =
                    entity::get_component(repulsive_dist_tuner, output()).unwrap_or(1.);
                let repulsive_strength: f32 =
                    entity::get_component(repulsive_str_tuner, output()).unwrap_or(1.);
                for (boid, pos) in &boids {
                    let mut repulsive_force = Vec3::ZERO;
                    for (oboid, opos) in &boids {
                        if oboid != boid
                            && opos.distance_squared(*pos) < repulsive_dist * repulsive_dist
                        {
                            repulsive_force += *pos - *opos;
                        }
                    }
                    if repulsive_force.length_squared() > 0. {
                        entity::mutate_component(*boid, boid_velocity(), move |v| {
                            *v += repulsive_force * repulsive_strength * dt;
                        });
                    }
                }
            });
    }

    // edge repulsion
    {
        query(translation())
            .requires(is_boid())
            .each_frame(move |boids| {
                let dt = delta_time();
                let edge_sqradius: f32 = 25.;
                let edge_strength: f32 = 19.;
                for (boid, pos) in &boids {
                    entity::mutate_component(*boid, boid_velocity(), move |v| {
                        if pos.x.abs() > edge_sqradius {
                            v.x -= pos.x.signum() * edge_strength * dt;
                        }
                        if pos.y.abs() > edge_sqradius {
                            v.y -= pos.y.signum() * edge_strength * dt;
                        }
                        if pos.z.abs() > edge_sqradius {
                            v.z -= pos.z.signum() * edge_strength * dt;
                        }
                    });
                }
            });
    }
}

fn mk_tuner(tuner_name: &str, starting_value: u32, max_value: u32) -> EntityId {
    Tuner {
        raw_value: starting_value as f32 / max_value as f32,
        tuner_max: max_value as f32,
        ..Tuner::suggested()
    }
    .make()
    .with(name(), tuner_name.to_string())
    .spawn()
}
