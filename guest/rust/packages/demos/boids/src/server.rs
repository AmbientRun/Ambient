use std::f32::consts::PI;

use ambient_api::{
    animation::{AnimationPlayerRef, PlayClipFromUrlNodeRef},
    core::{
        animation::components::{apply_animation_player, speed},
        app::components::name,
        camera::concepts::{
            PerspectiveInfiniteReverseCamera, PerspectiveInfiniteReverseCameraOptional,
        },
        model::components::model_from_url,
        network::components::no_sync,
        physics::components::{linear_velocity, plane_collider},
        primitives::components::{cube, quad},
        rendering::components::{
            cast_shadows, color, light_ambient, light_diffuse, transparency_group,
        },
        transform::components::{
            local_to_parent, local_to_world, lookat_target, lookat_up, rotation, scale, translation,
        },
    },
    prelude::*,
};

use packages::{
    dead_meets_lead_content::assets,
    this::components::*,
    tuners::{components::*, concepts::Tuner},
};

const MAXIMUM_SPAWNS_PER_FRAME: usize = 1;

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
    .with(is_boid_camera(), ())
    .with(lookat_target(), vec3(0., 0., -10.))
    .spawn();

    let _floor_collider = Entity::new()
        .with(plane_collider(), ())
        .with(translation(), Vec3::ZERO)
        .spawn();
    let floor = Entity::new()
        .with(quad(), ())
        .with(scale(), Vec3::splat(60.))
        .with(color(), vec4(0.1, 0.3, 0.15, 1.))
        .spawn();
    entity::add_child(
        floor,
        Entity::new()
            .with(translation(), vec3(0., 0., -0.001))
            .with(quad(), ())
            .with(scale(), Vec3::splat(2.))
            .with(color(), vec4(0.05, 0.15, 0.075, 1.))
            .with(local_to_parent(), Mat4::default())
            .spawn(),
    );

    let _sun = Entity::new()
        .with(light_ambient(), vec3(0.5, 0.5, 0.5))
        .with(light_diffuse(), vec3(0.75, 0.75, 0.75))
        .with(rotation(), Quat::from_rotation_x(PI * 0.25))
        .spawn();

    init_boids_logic(camera_ent, floor);

    println!("Hello, Ambient!");
}

fn make_boid(pos: Option<Vec3>, vel: Option<Vec3>) -> Entity {
    let starting_vel: Vec3 = match (vel, vel == Some(Vec3::ZERO)) {
        (None, _) | (_, true) => (random::<Vec2>() - 0.5).extend(0.) * 50. * 2.,
        (Some(vel), _) => vel,
    };
    let boid = Entity::new()
        .with(is_boid(), ())
        .with(linear_velocity(), starting_vel)
        .with(boid_neighbour_count(), 0)
        .with(local_to_world(), Mat4::default());

    match pos {
        None => boid,
        Some(starting_pos) => boid.with(translation(), starting_pos),
    }
}

fn init_boids_logic(camera_ent: EntityId, floor_ent: EntityId) {
    for _ in 0..100 {
        make_boid(None, None).spawn();
    }

    let quantity_min_tuner = mk_tuner("Min # Boids", (1, 21, 1001), true);
    let quantity_max_tuner = mk_tuner("Max # Boids", (1, 21, 1001), true);
    let size_tuner = mk_tuner("Size of Arena", (10, 10, 210), true);
    describe(
        quantity_min_tuner,
        "Auto-spawn boids to this minimum value.\n(Rate limited by MAXIMUM_SPAWNS_PER_FRAME)",
    );
    describe(quantity_max_tuner, "Despawn boids above max");
    describe(size_tuner, "Arena size. Boids attempt to stay inside.");

    let reproduction_rate_tuner = mk_tuner("Touching % Reproduce Rate/second", (0, 25, 100), true);
    let reproduction_neighbours_tuner = mk_tuner("Reproduce - Max neighbours", (0, 5, 50), false);
    let fighting_rate_tuner = mk_tuner("Touching % Kill Rate/second", (0, 5, 100), true);
    let birth_confetti_tuner = mk_tuner("Birth Particles", (0, 10, 100), true);
    let corpse_lifespan_tuner = mk_tuner("Corpse Lifespan", (0, 1, 10), false);
    describe(
        reproduction_rate_tuner,
        "Chance per second of a boid spawning a new boid\n(Only if near other boids)",
    );
    describe(
        reproduction_neighbours_tuner,
        "Overcrowded boids don't give birth",
    );
    describe(fighting_rate_tuner, "Chance to kill each nearby boid");
    describe(
        birth_confetti_tuner,
        "Semitransparent particles spawned per birth",
    );
    describe(
        corpse_lifespan_tuner,
        "Note; At 0 the corpse particle is still spawned, it is just deleted immediately.",
    );

    // global 'neighbour count opacity' value
    let neighbour_count_opacity_tuner = mk_tuner("Neighbour Count Opacity", (0, 100, 1), true);
    entity::add_component(
        neighbour_count_opacity_tuner,
        is_tuner_of_ncount_opacity(),
        (),
    );
    describe(
        neighbour_count_opacity_tuner,
        "Opacity of 'neighbour count' digit (at 0.00, disabled)",
    );

    // hidden boids param tuners - not important for benchmark
    let match_dist_tuner = mk_tuner("Match Range", (0, 10, 50), false);
    let posmatch_str_tuner = mk_tuner("Match Position (Coherence)", (0, 1, 10), false);
    let velmatch_str_tuner = mk_tuner("Match Velocity (Alignment)", (0, 5, 25), false);
    let repulsive_dist_tuner = mk_tuner("Touching Range", (0, 4, 10), false);
    let repulsive_str_tuner = mk_tuner("Touching Repel (Avoidance)", (0, 6, 20), false);

    // integer tuners.
    for int_tuner in [
        &quantity_min_tuner,
        &quantity_max_tuner,
        &size_tuner,
        &reproduction_neighbours_tuner,
        &birth_confetti_tuner,
    ] {
        entity::add_component(*int_tuner, is_int(), ());
    }

    // tuners that can be MEANINGFULLY, COMPUTATIONALLY disabled at 0.
    for disableable_tuner in [&neighbour_count_opacity_tuner] {
        entity::add_component(*disableable_tuner, is_nonpositive_off(), ());
    }

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
                    entity::despawn_recursive(boid);
                    left_to_remove -= 1;
                    if left_to_remove <= 0 {
                        break;
                    }
                }
                boid_count = max_quantity;
            }
            if boid_count < min_quantity {
                for _ in 0..(min_quantity - boid_count).min(MAXIMUM_SPAWNS_PER_FRAME) {
                    make_boid(None, None).spawn();
                    boid_count += 1;
                }
            }
        });
    }

    // basic velocity - speed limit, as well as turning to face
    {
        query((translation(), linear_velocity()))
            .requires(is_boid())
            .each_frame(|boids| {
                let dt = delta_time();
                let minspeed = 15.;
                let maxspeed = 30.;
                for (boid, (mut pos, mut vel)) in boids {
                    let dir = vel.normalize_or_zero();
                    if dir.length_squared() > 0.001 {
                        if vel.length_squared() < minspeed * minspeed {
                            vel = vel.normalize_or_zero() * minspeed;
                            entity::set_component(boid, linear_velocity(), vel);
                        }
                        if vel.length_squared() > maxspeed * maxspeed {
                            vel = vel.normalize() * maxspeed;
                            entity::set_component(boid, linear_velocity(), vel);
                        }
                        pos += vel * dt;
                        entity::add_component(boid, lookat_target(), pos + dir);
                        entity::set_component(boid, translation(), pos);
                    }
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

                        entity::mutate_component(*boid, linear_velocity(), move |v| {
                            *v += (perceived_center - *pos) * posmatch_strength * dt;
                        });
                    }
                }
            });
    }

    // velocity matching
    {
        let velmatch_dist_tuner = match_dist_tuner.clone();
        query((translation(), linear_velocity()))
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
                        entity::mutate_component(*boid, linear_velocity(), move |v| {
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
                let max_neighbours_for_reproduction: usize =
                    entity::get_component(reproduction_neighbours_tuner, output())
                        .unwrap_or(5.)
                        .round() as usize;
                let chance_reproduce: f32 =
                    entity::get_component(reproduction_rate_tuner, output()).unwrap_or(0.1)
                        * dt
                        * 0.01;
                let chance_fight: f32 =
                    entity::get_component(fighting_rate_tuner, output()).unwrap_or(0.1) * dt * 0.01;
                for (boid, pos) in &boids {
                    let mut repulsive_force = Vec3::ZERO;
                    let mut neighbours: usize = 0;
                    for (oboid, opos) in &boids {
                        if oboid != boid
                            && opos.distance_squared(*pos) < repulsive_dist * repulsive_dist
                        {
                            // yes we're touching
                            neighbours += 1;
                            repulsive_force += (*pos - *opos).normalize_or_zero();
                            if random::<f32>() < chance_fight {
                                entity::despawn_recursive(*oboid); // despawn my opponent >:)
                            }
                        }
                    }

                    if neighbours > 0
                        && neighbours < max_neighbours_for_reproduction
                        && random::<f32>() < chance_reproduce
                    {
                        make_boid(
                            Some(*pos + random::<Vec2>().extend(0.) * 2.),
                            entity::get_component(*boid, linear_velocity()),
                        )
                        .spawn();
                    }

                    if repulsive_force.length_squared() > 0. {
                        entity::mutate_component(*boid, linear_velocity(), move |v| {
                            *v += repulsive_force * repulsive_strength * dt;
                        });
                    }

                    if entity::exists(*boid) {
                        // TODO: separate neighbour count stuff out into another query. for now this is fine.
                        entity::add_component(*boid, boid_neighbour_count(), neighbours as u32);
                    }
                }
            });
    }

    // boids with more neighbours are... bigger? sure, whatever
    // no, does nothing
    {
        change_query(boid_neighbour_count())
            .track_change(boid_neighbour_count())
            .bind(|boids| {
                for (boid, bnct) in boids {
                    if let Some(model) = entity::get_component(boid, boid_model()) {
                        // entity::add_component(
                        //     model,
                        //     scale(),
                        //     Vec3::splat(2.50 + 0.25 * bnct as f32),
                        // );
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
                entity::add_component(floor_ent, scale(), Vec3::splat(edge_sqradius + 20.) * 2.); // scale ground quad
                for (boid, pos) in &boids {
                    entity::mutate_component(*boid, linear_velocity(), move |v| {
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

    // onspawn - give random position
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

    // onspawn - without lookat_up
    {
        spawn_query((translation(), linear_velocity()))
            .requires(is_boid())
            .excludes(lookat_up())
            .bind(move |lookless_boids| {
                for (boid, (pos, vel)) in lookless_boids {
                    entity::add_component(boid, lookat_up(), vec3(0., 0., 1.));
                    if vel != Vec3::ZERO {
                        entity::add_component(boid, lookat_target(), pos + vel);
                    }
                }
            });
    }

    // onspawn - add model and animation
    {
        spawn_query(())
            .requires((translation(), is_boid()))
            .bind(move |newboids| {
                for (newboid, _) in newboids {
                    let run_clip = PlayClipFromUrlNodeRef::new(assets::url(
                        "Data/Models/Units/Zombie1.x/animations/Run1.anim",
                    ));
                    let _idle_clip = PlayClipFromUrlNodeRef::new(assets::url(
                        "Data/Models/Units/Zombie1.x/animations/Idle1.anim",
                    ));
                    let anim_player = AnimationPlayerRef::new(&run_clip);
                    let model = Entity::new()
                        .with(translation(), Vec3::ZERO)
                        .with(
                            rotation(),
                            Quat::from_rotation_x(PI * -0.5) * Quat::from_rotation_z(PI * -0.5),
                        )
                        .with(model_from_url(), assets::url("Data/Models/Units/Zombie1.x"))
                        .with(scale(), Vec3::splat(3.0))
                        .with(local_to_parent(), Mat4::default())
                        .with(cast_shadows(), ())
                        .with(apply_animation_player(), anim_player.0)
                        .spawn();
                    entity::add_child(newboid, model);
                    entity::add_component(newboid, boid_model(), model);
                    entity::add_child(model, anim_player.0);

                    // entity::add_component(anim_player.0, speed(), random::<f32>() * 10.);
                    entity::add_component(run_clip.0.get_entity_id(), speed(), 2.0);
                    // ^speeds up animation playback, but not looping
                }
            });
    }

    // onspawn - make confetti
    {
        spawn_query(translation())
            .requires(is_boid())
            .bind(move |newboids| {
                let confetti_count = entity::get_component(birth_confetti_tuner, output())
                    .unwrap_or(0.)
                    .round() as usize;
                for (_newboid, pos) in newboids {
                    for _ in 0..confetti_count {
                        Entity::new()
                            .with(
                                translation(),
                                pos + (random::<Vec2>() - 0.5).extend(random::<f32>() * 0.5),
                            ) // anywhere inside the new cube's top half
                            .with(
                                linear_velocity(),
                                ((random::<Vec2>() - 0.5) * 10.).extend(random::<f32>() * 20.),
                            )
                            .with(cube(), ())
                            .with(transparency_group(), 1)
                            .with(color(), (random::<Vec3>() * 2.).extend(0.25))
                            .with(is_confetti(), ())
                            .with(rotation(), random::<Quat>().normalize())
                            .spawn();
                    }
                }
            });
    }

    // confetti movement
    {
        const CONFETTI_GRAVITY: f32 = 40.;
        const CONFETTI_DRAG: f32 = 0.05;
        query((translation(), linear_velocity()))
            .requires(is_confetti())
            .each_frame(|confettis| {
                let dt = delta_time();
                for (confetti, (pos, vel)) in confettis {
                    if pos.z <= 0. {
                        entity::despawn(confetti);
                    } else {
                        entity::mutate_component(confetti, linear_velocity(), |vel| {
                            *vel *= 1.00 - CONFETTI_DRAG * dt;
                            vel.z -= CONFETTI_GRAVITY * dt;
                        });
                        entity::mutate_component(confetti, translation(), move |pos| {
                            *pos += vel * dt;
                        });
                    }
                }
            });
    }

    // on despawn - make corpse, delete children (shouldn't this happen automatically though?)
    {
        despawn_query(translation())
            .requires(is_boid())
            .bind(move |deadboids| {
                for (_, pos) in deadboids {
                    Entity::new()
                        .with(translation(), pos + vec3(0., 0., 0.25))
                        .with(rotation(), random::<Quat>().normalize())
                        .with(color(), vec4(0.5, 0., 0., 1.))
                        .with(cube(), ())
                        .with(is_corpse(), ())
                        .with(corpse_age(), 0.)
                        .spawn();
                }
            });
    }

    // corpse animation
    {
        query(corpse_age())
            .requires(is_corpse())
            .each_frame(move |corpses| {
                let dt = delta_time();
                let corpse_lifespan =
                    entity::get_component(corpse_lifespan_tuner, output()).unwrap_or(1.);
                let age_delta_this_frame = {
                    if corpse_lifespan > 0.0001 {
                        dt / corpse_lifespan
                    } else {
                        1. // instant death
                    }
                };
                for (corpse, mut age) in corpses {
                    age += age_delta_this_frame;
                    if age < 1. {
                        entity::mutate_component(corpse, translation(), |pos| {
                            pos.z -= age_delta_this_frame
                        }); // corpses fall
                        entity::add_component(corpse, scale(), Vec3::splat(1. - age)); // corpses shrink
                        entity::add_component(corpse, corpse_age(), age);
                    } else {
                        entity::despawn(corpse);
                    }
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

fn describe(tuner: EntityId, desc: &str) {
    entity::add_component(tuner, description(), desc.to_string());
}
