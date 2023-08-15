use ambient_api::{
    core::{
        app::components::main_scene,
        camera::{
            components::aspect_ratio_from_window,
            concepts::make_perspective_infinite_reverse_camera,
        },
        messages::{Collision, Frame},
        physics::components::{
            angular_velocity, cube_collider, dynamic, linear_velocity, physics_controlled,
            visualize_collider,
        },
        prefab::components::prefab_from_url,
        primitives::components::cube,
        rendering::components::{cast_shadows, color},
        transform::{
            components::{lookat_target, rotation, scale, translation},
            concepts::make_transformable,
        },
    },
    prelude::*,
};

use embers::ambient_example_physics::{assets, messages::Bonk};

#[main]
pub async fn main() {
    let camera = Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with(main_scene(), ())
        .with(translation(), vec3(5., 5., 4.))
        .with(lookat_target(), vec3(0., 0., 0.))
        .spawn();

    let cube = Entity::new()
        .with_merge(make_transformable())
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
        .with_merge(make_transformable())
        .with(prefab_from_url(), assets::url("shape.glb"))
        .spawn();

    Collision::subscribe(move |msg| {
        println!("Bonk! {:?} collided", msg.ids);
        Bonk::new(cube, camera).send_client_broadcast_unreliable();
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
