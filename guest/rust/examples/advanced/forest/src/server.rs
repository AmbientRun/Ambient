use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        primitives::{
            tree_seed, tree_trunk_radius, tree_trunk_height, tree_branch_length, tree_branch_angle,
            tree_foliage_radius, tree_foliage_density, tree_trunk_segments, tree_branch_segments,
            tree_foliage_segments, quad
        },
        rendering::color,
        transform::{lookat_target, scale, translation},
    },
    concepts::{
        make_tree, make_perspective_infinite_reverse_camera, make_transformable,
    },
    prelude::*,
};

use rand::SeedableRng;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

#[main]
pub fn main() {

    // this seed initiates a PSRNG (pseudo randomness) to generate the same forest on all clients
    // also used for golden image testing :)
    let seed = 123456;

    let num_trees = 100;

    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with_default(main_scene())
        .with(translation(), vec3(365., 365., 62.))
        .with(lookat_target(), vec3(0., 0., 2.))
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with_default(quad())
        .with(scale(), Vec3::ONE * 500.)
        .with(color(), vec4(1., 0., 0., 1.))
        .spawn();


    // lets plant some trees :)
    for i in 0..num_trees {

        Entity::new()
            .with_merge(make_transformable())
            .with_merge(make_tree())
            .with(tree_seed(), seed  + i as i32 )
            .with(tree_trunk_radius(), gen_rn(seed + i as i32, 10.0, 20.0))
            .with(tree_trunk_height(), gen_rn(seed + i as i32, 50.0, 200.0))
            .with(tree_trunk_segments(), gen_rn(seed + i as i32, 4.0, 10.0) as u32)
            .with(tree_foliage_radius(), 15.0)
            .with(tree_foliage_segments(), 5)
            .with(tree_foliage_density(), 20)
            .with(scale(), Vec3::ONE * gen_rn(i as i32,  0.1,0.5))
            .with(translation(), vec3(
                gen_rn(seed + i as i32, 0.0, 250.0),
                gen_rn(seed + seed + i as i32, 0.0, 250.0),
                0.0,
            ))
            .with(color(), vec4(
                0.2,
                gen_rn(seed + i*i as i32,0.3,0.9),
                gen_rn(seed + (i*i*i) as i32,0.2,0.3),
                1.0,
            ))
            .spawn();
    }
}

pub fn gen_rn(seed: i32, min: f32, max: f32) -> f32 {
    let mut state: u32 = 0;
    let mut rng = ChaCha8Rng::seed_from_u64(seed as u64);
    let mut n = rng.gen::<f32>();
    n = n * (max - min) + min;
    n
}