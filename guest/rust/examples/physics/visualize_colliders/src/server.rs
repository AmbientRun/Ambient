use ambient_api::{
    core::{
        app::components::name,
        camera::concepts::{
            PerspectiveInfiniteReverseCamera, PerspectiveInfiniteReverseCameraOptional,
        },
        physics::{
            components::{cube_collider, sphere_collider, visualize_collider},
            concepts::CharacterController,
        },
        primitives::{
            components::{cube, quad},
            concepts::{Capsule, Sphere},
        },
        transform::{
            components::{lookat_target, mesh_to_local, translation},
            concepts::{Transformable, TransformableOptional},
        },
    },
    prelude::*,
};
use packages::orbit_camera::concepts::{OrbitCamera, OrbitCameraOptional};

#[main]
pub fn main() {
    main2();
}
fn main2() {
    OrbitCamera {
        is_orbit_camera: (),
        optional: OrbitCameraOptional {
            camera_angle: Some(vec2(135f32.to_radians(), 35f32.to_radians())),
            camera_distance: Some(10.),
            ..default()
        },
    }
    .spawn();

    Entity::new()
        .with_merge(Transformable {
            local_to_world: Mat4::IDENTITY,
            optional: TransformableOptional {
                translation: Some(Vec3::ZERO),
                ..default()
            },
        })
        .with(cube(), ())
        .with(cube_collider(), Vec3::ONE)
        .with(visualize_collider(), ())
        .spawn();

    Entity::new()
        .with_merge(Transformable {
            local_to_world: Mat4::IDENTITY,
            optional: TransformableOptional {
                translation: Some(Vec3::Y * 2.),
                ..default()
            },
        })
        .with_merge(Sphere::suggested())
        .with(sphere_collider(), 0.5)
        .with(visualize_collider(), ())
        .spawn();

    Entity::new()
        .with(name(), "Character collider".to_string())
        .with_merge(Transformable {
            local_to_world: Mat4::IDENTITY,
            optional: TransformableOptional {
                translation: Some(Vec3::X * 2.),
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
        // The capsule mesh is centered around 0, whereas the collider is centered to "stand on the ground"
        .with(mesh_to_local(), Mat4::from_translation(Vec3::Z))
        .with_merge(CharacterController {
            character_controller_height: 2.,
            character_controller_radius: 0.5,
            physics_controlled: (),
        })
        .with(visualize_collider(), ())
        .spawn();
}
