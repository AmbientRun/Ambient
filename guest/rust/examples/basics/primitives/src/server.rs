use ambient_api::{
    core::{
        primitives::{
            components::{cube, quad},
            concepts::{Capsule, Sphere, Torus},
        },
        rendering::components::{color, double_sided},
        transform::concepts::{Transformable, TransformableOptional},
    },
    prelude::*,
};
use packages::orbit_camera::concepts::{OrbitCamera, OrbitCameraOptional};

#[main]
pub fn main() {
    OrbitCamera {
        is_orbit_camera: (),
        optional: OrbitCameraOptional {
            camera_distance: Some(10.),
            ..default()
        },
    }
    .spawn();

    Entity::new()
        .with_merge(Transformable {
            local_to_world: Mat4::IDENTITY,
            optional: TransformableOptional {
                scale: Some(Vec3::ONE * 10.),
                ..default()
            },
        })
        .with(quad(), ())
        .with(double_sided(), true)
        .with(color(), vec4(1., 0., 0., 1.))
        .spawn();

    Entity::new()
        .with_merge(Transformable {
            local_to_world: Mat4::IDENTITY,
            optional: TransformableOptional {
                translation: Some(vec3(2., 0., 0.5)),
                ..default()
            },
        })
        .with(cube(), ())
        .with(color(), vec4(0., 1., 0., 1.))
        .spawn();

    Entity::new()
        .with_merge(Transformable {
            local_to_world: Mat4::IDENTITY,
            optional: TransformableOptional {
                translation: Some(vec3(0., 2., 0.5)),
                ..default()
            },
        })
        .with_merge(Sphere {
            sphere: (),
            sphere_radius: 1.0,
            sphere_sectors: 12,
            sphere_stacks: 6,
        })
        .with(color(), vec4(0., 0., 1., 1.))
        .spawn();

    Entity::new()
        .with_merge(Transformable {
            local_to_world: Mat4::IDENTITY,
            optional: TransformableOptional {
                translation: Some(vec3(2., 2., 0.25)),
                ..default()
            },
        })
        .with_merge(Sphere {
            sphere: (),
            sphere_radius: 0.5,
            sphere_sectors: 36,
            sphere_stacks: 18,
        })
        .with(color(), vec4(1., 1., 0., 1.))
        .spawn();

    Entity::new()
        .with_merge(Transformable {
            local_to_world: Mat4::IDENTITY,
            optional: TransformableOptional {
                translation: Some(vec3(-2.0, 2.0, 1.0)),
                ..default()
            },
        })
        .with_merge(Capsule {
            capsule: (),
            capsule_radius: 0.5,
            capsule_half_height: 0.5,
            capsule_rings: 0,
            capsule_latitudes: 16,
            capsule_longitudes: 32,
        })
        .with(color(), vec4(1.0, 0.25, 0.0, 1.0))
        .spawn();

    Entity::new()
        .with_merge(Transformable {
            local_to_world: Mat4::IDENTITY,
            optional: TransformableOptional {
                translation: Some(vec3(-2.0, 0.0, 0.5)),
                ..default()
            },
        })
        .with_merge(Capsule {
            capsule: (),
            capsule_radius: 0.25,
            capsule_half_height: 0.25,
            capsule_rings: 0,
            capsule_latitudes: 16,
            capsule_longitudes: 32,
        })
        .with(color(), vec4(1.0, 0.0, 0.25, 1.0))
        .spawn();

    Entity::new()
        .with_merge(Transformable {
            local_to_world: Mat4::IDENTITY,
            optional: TransformableOptional {
                translation: Some(vec3(0.0, -2.0, 0.5)),
                ..default()
            },
        })
        .with_merge(Torus {
            torus: (),
            torus_inner_radius: 0.25,
            torus_outer_radius: 0.5,
            torus_slices: 32,
            torus_loops: 16,
        })
        .with(color(), vec4(0.0, 1.0, 0.25, 1.0))
        .spawn();
}
