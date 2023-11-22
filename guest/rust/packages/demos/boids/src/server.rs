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

use packages::this::components::*;
use packages::tuners::{components::output, concepts::Tuner};

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
        Entity::new()
            .with(translation(), random::<Vec3>() * 5.)
            .with(cube(), ())
            .with(is_boid(), ())
            .with(boid_velocity(), random::<Vec3>() * 10.)
            .spawn();
    }

    init_boids_logic();

    println!("Hello, Ambient!");
}

fn init_boids_logic() {
    let posmatch_dist_tuner = Tuner {
        tuner_max: 10. * 2.,
        ..Tuner::suggested()
    }
    .make()
    .with(name(), "Pos Dist".into())
    .spawn();

    let posmatch_str_tuner = Tuner {
        tuner_max: 2. * 2.,
        ..Tuner::suggested()
    }
    .make()
    .with(name(), "Pos Strength".into())
    .spawn();

    let repulsive_dist_tuner = Tuner {
        tuner_max: 2. * 2.,
        ..Tuner::suggested()
    }
    .make()
    .with(name(), "Rep Dist".into())
    .spawn();

    let repulsive_str_tuner = Tuner {
        raw_value: 0.1,
        tuner_max: 2. * 2.,
        ..Tuner::suggested()
    }
    .make()
    .with(name(), "Rep Strength".into())
    .spawn();

    let velmatch_dist_tuner = Tuner {
        tuner_max: 10. * 2.,
        ..Tuner::suggested()
    }
    .make()
    .with(name(), "Velmatch Dist".into())
    .spawn();

    let velmatch_str_tuner = Tuner {
        tuner_max: 2. * 2.,
        ..Tuner::suggested()
    }
    .make()
    .with(name(), "Velmatch Strength".into())
    .spawn();

    query((translation(), boid_velocity()))
        .requires(is_boid())
        .each_frame(|boids| {
            let dt = delta_time();
            for (boid, (pos, vel)) in boids {
                entity::add_component(boid, translation(), pos + vel * dt);
            }
        });

    // to center
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
                    if oboid != boid && opos.distance_squared(*pos) < posmatch_dist * posmatch_dist
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

    // repulsion
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

    // velocity matching
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
                    if oboid != boid && opos.distance_squared(*pos) < velmatch_dist * velmatch_dist
                    {
                        total_velocity += *ovel;
                        bns += 0;
                    }
                }
                if bns > 0 {
                    total_velocity /= bns as f32;
                    entity::mutate_component(*boid, boid_velocity(), move |v| {
                        *v += total_velocity * velmatch_strength * dt;
                    });
                }
            }
        });
}
