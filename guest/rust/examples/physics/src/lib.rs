use kiwi_api::{
    components::core::{
        camera::{aspect_ratio_from_window, perspective_infinite_reverse},
        ecs::ids,
        game_objects::player_camera,
        object::object_from_url,
        physics::{angular_velocity, box_collider, dynamic, linear_velocity, physics_controlled},
        primitives::cube,
        rendering::color,
        transform::{lookat_center, rotation, scale, translation},
    },
    prelude::*,
};

#[main]
pub async fn main() -> EventResult {
    entity::game_object_base()
        .with_default(player_camera())
        .with(translation(), vec3(5., 5., 4.))
        .with(lookat_center(), vec3(0., 0., 0.))
        .with(perspective_infinite_reverse(), ())
        .with(aspect_ratio_from_window(), ())
        .spawn();

    let cube = entity::game_object_base()
        .with_default(cube())
        .with(box_collider(), vec3(1., 1., 1.))
        .with(dynamic(), true)
        .with_default(physics_controlled())
        .with(translation(), vec3(0., 0., 5.))
        .with(scale(), vec3(0.5, 0.5, 0.5))
        .spawn();

    entity::game_object_base()
        .with(object_from_url(), "assets/Shape.glb".to_string())
        .spawn();

    on(event::COLLISION, |c| {
        // TODO: play a sound instead
        println!("Bonk! {:?} collided", c.get(ids()).unwrap());
        EventOk
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
            Components::new()
                .with(translation(), vec3(0., 0., 5.))
                .with(rotation(), Quat::IDENTITY)
                .with(linear_velocity(), new_linear_velocity)
                .with(angular_velocity(), new_angular_velocity)
                .with(color(), random::<Vec3>().extend(1.)),
        );
    }
}
