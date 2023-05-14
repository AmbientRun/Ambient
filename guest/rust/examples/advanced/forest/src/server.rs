use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        primitives::{
            tree_trunk_radius, tree_trunk_height, tree_branch_length, tree_branch_angle,
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

#[main]
pub fn main() {
    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with_default(main_scene())
        .with(translation(), vec3(150., 155., 38.))
        .with(lookat_target(), vec3(0., 0., 2.))
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with_default(quad())
        .with(scale(), Vec3::ONE * 10.)
        .with(color(), vec4(1., 0., 0., 1.))
        .spawn();

    let num_trees = 30;

    for _ in 0..num_trees {
        let _tree_trunk_radius = gen_range(1.0,2.0);
        let _tree_trunk_height = gen_range(5.0,11.0);
        let _tree_trunk_segments = 10;//gen_range(8.0,12.0);
        let _tree_branch_length = gen_range(0.4,0.7);
        let _tree_branch_angle = gen_range(0.2,0.5);
        let _tree_branch_segments = gen_range(1.0,3.0);
        let _tree_foliage_radius = gen_range(2.0,3.0);
        let _tree_foliage_segments = gen_range(3.0,5.0);
        let _tree_foliage_density = gen_range(3.0,5.0);
        let _tree_color = vec4(
            gen_range(0.6,0.8),
            gen_range(0.2,0.4),
            gen_range(0.1,0.3),
            1.0,
        );
        let _tree_translation = vec3(
            random::<f32>()*150.0,
            random::<f32>()*150.0,
            0.0,
        );

        Entity::new()
            .with_merge(make_transformable())
            .with_merge(make_tree())
            .with(tree_trunk_radius(), _tree_trunk_radius)
            .with(tree_trunk_height(), _tree_trunk_height)
            .with(tree_trunk_segments(), _tree_trunk_segments as u32)
            .with(tree_branch_length(), _tree_branch_length)
            .with(tree_branch_angle(), _tree_branch_angle)
            .with(tree_branch_segments(), _tree_branch_segments as u32)
            .with(tree_foliage_radius(), _tree_foliage_radius)
            .with(tree_foliage_segments(), _tree_foliage_segments as u32)
            .with(tree_foliage_density(), _tree_foliage_density as u32)
            .with(translation(), _tree_translation)
            .with(color(), _tree_color)
            .spawn();
    }
}

pub fn gen_range(min:f32, max:f32) -> f32 {
    (random::<f32>() +min) * max
}

