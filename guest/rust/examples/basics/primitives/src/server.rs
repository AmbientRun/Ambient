use ambient_api::{
    core::{
        app::components::main_scene,
        camera::{
            components::aspect_ratio_from_window, concepts::make_PerspectiveInfiniteReverseCamera,
        },
        primitives::{
            components::{
                capsule_half_height, capsule_latitudes, capsule_longitudes, capsule_radius,
                capsule_rings, cube, quad, sphere_radius, sphere_sectors, sphere_stacks,
                torus_inner_radius, torus_loops, torus_outer_radius, torus_slices,
            },
            concepts::{make_Capsule, make_Sphere, make_Torus},
        },
        rendering::components::color,
        transform::{
            components::{lookat_target, scale, translation},
            concepts::make_Transformable,
        },
    },
    prelude::*,
};

#[main]
pub fn main() {
    Entity::new()
        .with_merge(make_PerspectiveInfiniteReverseCamera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with(main_scene(), ())
        .with(translation(), vec3(5., 5., 6.))
        .with(lookat_target(), vec3(0., 0., 2.))
        .spawn();

    Entity::new()
        .with_merge(make_Transformable())
        .with(quad(), ())
        .with(scale(), Vec3::ONE * 10.)
        .with(color(), vec4(1., 0., 0., 1.))
        .spawn();

    Entity::new()
        .with_merge(make_Transformable())
        .with(cube(), ())
        .with(translation(), vec3(2., 0., 0.5))
        .with(scale(), Vec3::ONE)
        .with(color(), vec4(0., 1., 0., 1.))
        .spawn();

    Entity::new()
        .with_merge(make_Transformable())
        .with_merge(make_Sphere())
        .with(sphere_radius(), 1.0)
        .with(sphere_sectors(), 12)
        .with(sphere_stacks(), 6)
        .with(translation(), vec3(0., 2., 0.5))
        .with(color(), vec4(0., 0., 1., 1.))
        .spawn();

    Entity::new()
        .with_merge(make_Transformable())
        .with_merge(make_Sphere())
        .with(translation(), vec3(2., 2., 0.25))
        .with(color(), vec4(1., 1., 0., 1.))
        .spawn();

    Entity::new()
        .with_merge(make_Transformable())
        .with_merge(make_Capsule())
        .with(translation(), vec3(-2.0, 2.0, 1.0))
        .with(color(), vec4(1.0, 0.25, 0.0, 1.0))
        .spawn();

    Entity::new()
        .with_merge(make_Transformable())
        .with_merge(make_Capsule())
        .with(capsule_radius(), 0.25)
        .with(capsule_half_height(), 0.25)
        .with(capsule_rings(), 0)
        .with(capsule_latitudes(), 16)
        .with(capsule_longitudes(), 32)
        .with(translation(), vec3(-2.0, 0.0, 0.5))
        .with(color(), vec4(1.0, 0.0, 0.25, 1.0))
        .spawn();

    Entity::new()
        .with_merge(make_Transformable())
        .with_merge(make_Torus())
        .with(torus_inner_radius(), 0.25)
        .with(torus_outer_radius(), 0.5)
        .with(torus_slices(), 32)
        .with(torus_loops(), 16)
        .with(translation(), vec3(0.0, -2.0, 0.5))
        .with(color(), vec4(0.0, 1.0, 0.25, 1.0))
        .spawn();
}
