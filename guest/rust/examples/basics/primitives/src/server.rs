use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        primitives::{
            capsule_half_height, capsule_latitudes, capsule_longitudes, capsule_radius,
            capsule_rings, cube, quad, sphere_radius, sphere_sectors, sphere_stacks,
            torus_inner_radius, torus_loops, torus_outer_radius, torus_slices,
        },
        rendering::color,
        transform::{lookat_target, scale, translation},
    },
    concepts::{
        make_capsule, make_perspective_infinite_reverse_camera, make_sphere, make_torus,
        make_transformable,
    },
    prelude::*,
};

#[main]
pub fn main() {
    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with_default(main_scene())
        .with(translation(), vec3(5., 5., 6.))
        .with(lookat_target(), vec3(0., 0., 2.))
        .spawn();
    //
    // Entity::new()
    //     .with_merge(make_transformable())
    //     .with_default(quad())
    //     .with(scale(), Vec3::ONE * 10.)
    //     .with(color(), vec4(1., 0., 0., 1.))
    //     .spawn();

    // Entity::new()
    //     .with_merge(make_transformable())
    //     .with_default(cube())
    //     .with(translation(), vec3(2., 0., 0.5))
    //     .with(scale(), Vec3::ONE)
    //     .with(color(), vec4(0., 1., 0., 1.))
    //     .spawn();

    // Entity::new()
    //     .with_merge(make_transformable())
    //     .with_merge(make_sphere())
    //     .with(sphere_radius(), 1.)
    //     .with(sphere_sectors(), 12)
    //     .with(sphere_stacks(), 6)
    //     .with(translation(), vec3(0., 2., 1.0))
    //     .with(color(), vec4(1., 0., 0., 1.))
    //     .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with_merge(make_sphere())
        .with(sphere_radius(), 1.)
        .with(sphere_sectors(), 12)
        .with(sphere_stacks(), 6)
        .with(translation(), vec3(0., 2., 0.75))
        .with(color(), vec4(0., 1., 0., 1.))
        .spawn();

    // Entity::new()
    //     .with_merge(make_transformable())
    //     .with_merge(make_sphere())
    //     .with(sphere_radius(), 1.)
    //     .with(sphere_sectors(), 12)
    //     .with(sphere_stacks(), 6)
    //     .with(translation(), vec3(0., 2., 0.5))
    //     .with(color(), vec4(0., 0., 1., 1.))
    //     .spawn();

    // Entity::new()
    //     .with_merge(make_transformable())
    //     .with_merge(make_sphere())
    //     .with(translation(), vec3(2., 2., 0.25))
    //     .with(color(), vec4(1., 1., 0., 1.))
    //     .spawn();
    //
    // Entity::new()
    //     .with_merge(make_transformable())
    //     .with_merge(make_capsule())
    //     .with(translation(), vec3(-2.0, 2.0, 1.0))
    //     .with(color(), vec4(1.0, 0.25, 0.0, 1.0))
    //     .spawn();
    //
    // Entity::new()
    //     .with_merge(make_transformable())
    //     .with_merge(make_capsule())
    //     .with(capsule_radius(), 0.25)
    //     .with(capsule_half_height(), 0.25)
    //     .with(capsule_rings(), 0)
    //     .with(capsule_latitudes(), 16)
    //     .with(capsule_longitudes(), 32)
    //     .with(translation(), vec3(-2.0, 0.0, 0.5))
    //     .with(color(), vec4(1.0, 0.0, 0.25, 1.0))
    //     .spawn();
    //
    // Entity::new()
    //     .with_merge(make_transformable())
    //     .with_merge(make_torus())
    //     .with(torus_inner_radius(), 0.25)
    //     .with(torus_outer_radius(), 0.5)
    //     .with(torus_slices(), 32)
    //     .with(torus_loops(), 16)
    //     .with(translation(), vec3(0.0, -2.0, 0.5))
    //     .with(color(), vec4(0.0, 1.0, 0.25, 1.0))
    //     .spawn();
}
