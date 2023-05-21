use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        primitives::{
            sphere_radius, sphere_sectors, sphere_stacks,
            tree_foliage_density, tree_foliage_radius, tree_foliage_segments, tree_seed,
            tree_trunk_height, tree_trunk_radius, tree_trunk_segments,
        },
        physics::{
            character_controller_height, character_controller_radius, physics_controlled,
            plane_collider, sphere_collider,
        },
        player::{player, user_id},
        rendering::{color, fog_density, light_diffuse, sky, sun, water},
        transform::{lookat_target, scale, translation},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable, make_tree, make_sphere},
    prelude::*,
};

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

#[main]
pub fn main() {
    // this seed initiates a PSRNG (pseudo randomness) to generate the same forest on all clients
    // also used for golden image testing :)
    let seed = 123456;

    let num_trees = 100;

    // camera
    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with_default(main_scene())
        .with(translation(), vec3(365., 365., 62.))
        .with(lookat_target(), vec3(0., 0., 2.))
        .spawn();

    // ground
    Entity::new()
        .with_merge(make_transformable())
        .with_merge(make_sphere())
        .with(sphere_collider(), 0.5)
        .with(sphere_radius(), 1000.0)
        .with(sphere_sectors(), 24)
        .with(sphere_stacks(), 24)
        .with(translation(), vec3(80., 80., -970.))
        .with(color(), vec4(0.4, 0.2, 0.2, 1.))
        .spawn();

    // ocean
    Entity::new()
        .with_merge(make_transformable())
        .with_default(water())
        .with(scale(), Vec3::ONE * 1000.)
        .with(translation(), vec3(0., 0., 100.))
        .spawn();

    // sun, light, fog
    Entity::new()
        .with_merge(make_transformable())
        .with_default(sun())
        .with(rotation(), Quat::from_rotation_y(-45_f32.to_radians()))
        .with(light_diffuse(), Vec3::ONE)
        .with(fog_density(), 0.)
        .with_default(main_scene())
        .spawn();

    // sky
    Entity::new()
        .with_merge(make_transformable())
        .with_default(sky()).spawn();

    // lets plant some trees :)
    for i in 0..num_trees {
        Entity::new()
            .with_merge(make_transformable())
            .with_merge(make_tree())
            .with_default(cast_shadows())
            .with(tree_seed(), seed + i)
            .with(tree_trunk_radius(), gen_rn(seed + i, 10.0, 20.0))
            .with(tree_trunk_height(), gen_rn(seed + i, 50.0, 200.0))
            .with(tree_trunk_segments(), gen_rn(seed + i, 4.0, 10.0) as u32)
            .with(tree_foliage_radius(), 15.0)
            .with(tree_foliage_segments(), 5)
            .with(tree_foliage_density(), 20)
            .with(scale(), Vec3::ONE * gen_rn(i, 0.1, 0.5))
            .with(
                translation(),
                vec3(
                    gen_rn(seed + i, 0.0, 250.0),
                    gen_rn(seed + seed + i, 0.0, 250.0),
                    0.0,
                ),
            )
            .with(
                color(),
                vec4(
                    0.2,
                    gen_rn(seed + i * i, 0.3, 0.9),
                    gen_rn(seed + i * i * i, 0.2, 0.3),
                    1.0,
                ),
            )
            .spawn();
    }
}

pub fn gen_rn(seed: i32, min: f32, max: f32) -> f32 {
    let mut rng = ChaCha8Rng::seed_from_u64(seed as u64);
    rng.gen_range(min..max)
}
