use ambient_api::{
    core::{
        app::components::name,
        camera::concepts::{
            Camera, PerspectiveInfiniteReverseCamera, PerspectiveInfiniteReverseCameraOptional,
        },
        network::components::no_sync,
        primitives::components::{cube, quad},
        transform::components::{lookat_target, translation},
    },
    prelude::*,
};

use packages::{
    this::components::*,
    tuners::{components::*, concepts::Tuner},
};

#[main]
pub fn main() {
    let camera_ent = PerspectiveInfiniteReverseCamera {
        optional: PerspectiveInfiniteReverseCameraOptional {
            aspect_ratio_from_window: Some(entity::resources()),
            main_scene: Some(()),
            translation: Some(Vec3::ONE * 70.),
            ..default()
        },
        ..PerspectiveInfiniteReverseCamera::suggested()
    }
    .make()
    .with(lookat_target(), vec3(0., 0., -10.))
    .spawn();

    init_boids_logic(camera_ent);

    println!("Hello, Ambient!");
}

fn spawn_boid() {
    Entity::new()
        .with(cube(), ())
        .with(is_boid(), ())
        .with(
            boid_velocity(),
            (random::<Vec2>() - 0.5).extend(0.) * 50. * 2.,
            // (random::<Vec3>() - 0.5) * 50. * 2.,
        )
        .spawn();
}

fn spawn_boid_at(pos: Vec3, vel: Option<Vec3>) {
    Entity::new()
        .with(translation(), pos)
        .with(cube(), ())
        .with(is_boid(), ())
        .with(
            boid_velocity(),
            match vel {
                None => (random::<Vec2>() - 0.5).extend(0.) * 50. * 2.,
                Some(vel) => vel,
            },
            // (random::<Vec3>() - 0.5) * 50. * 2.,
        )
        .spawn();
}

fn init_boids_logic(camera_ent: EntityId) {
    for _ in 0..100 {
        spawn_boid();
    }

    let quantity_min_tuner = mk_tuner("Min # Boids", (1, 101, 1001), true);
    let quantity_max_tuner = mk_tuner("Max # Boids", (1, 201, 1001), true);
    let size_tuner = mk_tuner("Size of Arena", (10, 60, 210), true);
    let match_dist_tuner = mk_tuner("Match Range", (0, 10, 50), false);
    let posmatch_str_tuner = mk_tuner("Match Position (Coherence)", (0, 1, 10), false);
    let velmatch_str_tuner = mk_tuner("Match Velocity (Alignment)", (0, 5, 25), false);
    let repulsive_dist_tuner = mk_tuner("Touching Range", (0, 4, 10), false);
    let repulsive_str_tuner = mk_tuner("Touching Repel (Avoidance)", (0, 6, 20), false);

    let reproduction_rate_tuner = mk_tuner("Touching % Reproduce Rate/second", (0, 5, 100), true);
    let fighting_rate_tuner = mk_tuner("Touching % Kill Rate/second", (0, 5, 100), true);

    // boids quantity
    {
        query(()).requires(is_boid()).each_frame(move |mut boids| {
            let min_quantity: usize = entity::get_component(quantity_min_tuner, output())
                .unwrap_or(1.)
                .round() as usize;
            let max_quantity: usize = entity::get_component(quantity_max_tuner, output())
                .unwrap_or(1.)
                .round() as usize;
            let mut boid_count: usize = boids.len();
            if boid_count > max_quantity {
                boids.shuffle(&mut thread_rng());
                let mut left_to_remove = boid_count - max_quantity;
                for (boid, _) in boids {
                    entity::despawn(boid);
                    left_to_remove -= 1;
                    if left_to_remove <= 0 {
                        break;
                    }
                }
                boid_count = max_quantity;
            }
            if boid_count < min_quantity {
                for _ in 0..min_quantity - boid_count {
                    spawn_boid();
                }
                // boid_count = min_quantity; // not used after this
            }
        });
    }

    // basic velocity and speed limit
    {
        query((translation(), boid_velocity()))
            .requires(is_boid())
            .each_frame(|boids| {
                let dt = delta_time();
                let minspeed = 15.;
                let maxspeed = 30.;
                for (boid, (pos, mut vel)) in boids {
                    if vel.length_squared() < minspeed * minspeed {
                        vel = vel.normalize_or_zero() * minspeed;
                        entity::set_component(boid, boid_velocity(), vel);
                    }
                    if vel.length_squared() > maxspeed * maxspeed {
                        vel = vel.normalize() * maxspeed;
                        entity::set_component(boid, boid_velocity(), vel);
                    }
                    entity::set_component(boid, translation(), pos + vel * dt);
                }
            });
    }

    // to center
    {
        let posmatch_dist_tuner = match_dist_tuner.clone();

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
                            bns += 1;
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

    // touching
    {
        query(translation())
            .requires(is_boid())
            .each_frame(move |boids| {
                let dt = delta_time();
                let repulsive_dist: f32 =
                    entity::get_component(repulsive_dist_tuner, output()).unwrap_or(1.);
                let repulsive_strength: f32 =
                    entity::get_component(repulsive_str_tuner, output()).unwrap_or(1.);
                let chance_reproduce: f32 =
                    entity::get_component(reproduction_rate_tuner, output()).unwrap_or(0.1)
                        * dt
                        * 0.01;
                let chance_fight: f32 =
                    entity::get_component(fighting_rate_tuner, output()).unwrap_or(0.1) * dt * 0.01;
                for (boid, pos) in &boids {
                    let mut repulsive_force = Vec3::ZERO;
                    for (oboid, opos) in &boids {
                        if oboid != boid
                            && opos.distance_squared(*pos) < repulsive_dist * repulsive_dist
                        {
                            // yes we're touching
                            repulsive_force += (*pos - *opos).normalize_or_zero();
                            if random::<f32>() < chance_reproduce {
                                spawn_boid_at(
                                    (*pos + *opos) * 0.5,
                                    entity::get_component(*boid, boid_velocity()),
                                );
                            }
                            if random::<f32>() < chance_fight {
                                entity::despawn(*boid);
                            }
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
                let edge_sqradius: f32 =
                    entity::get_component(*&size_tuner, output()).unwrap_or(10.);
                let edge_strength: f32 = 19.;
                entity::add_component(camera_ent, translation(), Vec3::splat(edge_sqradius + 20.)); // move camera out according to size
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

    // onspawn
    {
        spawn_query(())
            .requires(is_boid())
            .excludes(translation())
            .bind(move |newboids| {
                let edge_sqradius: f32 =
                    entity::get_component(*&size_tuner, output()).unwrap_or(10.);
                for (newboid, _) in newboids {
                    entity::add_component(
                        newboid,
                        translation(),
                        (random::<Vec2>() - 0.5).extend(0.) * edge_sqradius * 2.,
                    );
                }
            });
    }
}

fn mk_tuner(tuner_name: &str, min_starting_max: (u32, u32, u32), tuning_enabled: bool) -> EntityId {
    let (min_value, starting_value, max_value) = min_starting_max;
    let mut tuner = Tuner {
        tuner_min: min_value as f32,
        raw_value: (starting_value as f32 - min_value as f32)
            / (max_value as f32 - min_value as f32),
        tuner_max: max_value as f32,
        ..Tuner::suggested()
    }
    .make()
    .with(name(), tuner_name.to_string());
    if !tuning_enabled {
        tuner = tuner.with(no_sync(), ());
    }
    tuner.spawn()
}
