use ambient_api::{
    core::{
        messages::{Collision, Frame},
        physics::components::{
            angular_velocity, cube_collider, dynamic, linear_velocity, physics_controlled,
            visualize_collider,
        },
        prefab::components::prefab_from_url,
        primitives::components::cube,
        rendering::components::{cast_shadows, color},
        transform::components::{rotation, scale, translation},
    },
    prelude::*,
};

use packages::{
    orbit_camera::concepts::{OrbitCamera, OrbitCameraOptional},
    this::{assets, messages::Bonk},
};

#[main]
pub async fn main() {
    let camera = OrbitCamera {
        is_orbit_camera: (),
        lookat_target: Vec3::ZERO,
        optional: OrbitCameraOptional {
            camera_distance: Some(7.5),
            camera_angle: Some(vec2(45f32.to_radians(), 45f32.to_radians())),
            ..default()
        },
    }
    .make()
    .spawn();

    let cube = Entity::new()
        .with(cube(), ())
        .with(visualize_collider(), ())
        .with(physics_controlled(), ())
        .with(cast_shadows(), ())
        .with(linear_velocity(), Vec3::ZERO)
        .with(angular_velocity(), Vec3::ZERO)
        .with(cube_collider(), Vec3::ONE)
        .with(dynamic(), true)
        .with(translation(), vec3(0., 0., 5.))
        .with(rotation(), Quat::IDENTITY)
        .with(scale(), vec3(0.5, 0.5, 0.5))
        .with(color(), Vec4::ONE)
        .spawn();

    Entity::new()
        .with(prefab_from_url(), assets::url("shape.glb"))
        .spawn();

    Collision::subscribe(move |msg| {
        println!("Bonk! {:?} collided", msg.ids);
        Bonk {
            emitter: cube,
            listener: camera,
        }
        .send_client_broadcast_unreliable();
    });

    Frame::subscribe(move |_| {
        for hit in physics::raycast(Vec3::Z * 20., -Vec3::Z) {
            if hit.entity == cube {
                println!("The raycast hit the cube: {hit:?}");
            }
        }
    });

    loop {
        let max_linear_velocity = 2.5;
        let max_angular_velocity = 360.0f32.to_radians();

        sleep(5.).await;

        let new_linear_velocity = (random::<Vec3>() - 0.5) * 2. * max_linear_velocity;
        let new_angular_velocity = (random::<Vec3>() - 0.5) * 2. * max_angular_velocity;
        println!("And again! Linear velocity: {new_linear_velocity:?} | Angular velocity: {new_angular_velocity:?}");
        entity::set_components(
            cube,
            Entity::new()
                .with(translation(), vec3(0., 0., 5.))
                .with(rotation(), Quat::IDENTITY)
                .with(linear_velocity(), new_linear_velocity)
                .with(angular_velocity(), new_angular_velocity)
                .with(color(), random::<Vec3>().extend(1.)),
        );
    }
}
