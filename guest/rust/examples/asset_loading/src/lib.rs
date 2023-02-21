use ambient_api::{
    components::core::{
        game_objects::player_camera,
        object::object_from_url,
        transform::{lookat_center, rotation, translation},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    prelude::*,
};

#[main]
pub async fn main() -> EventResult {
    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with_default(player_camera())
        .with(translation(), vec3(5., 5., 4.))
        .with(lookat_center(), vec3(0., 0., 0.))
        .spawn();

    let cube_id = Entity::new()
        .with_merge(make_transformable())
        .with(object_from_url(), asset_url("assets/Cube.glb").unwrap())
        .with(components::is_the_best(), true)
        .spawn();

    on(event::FRAME, move |_| {
        entity::set_component(
            cube_id,
            rotation(),
            Quat::from_axis_angle(Vec3::X, time().sin()),
        );

        EventOk
    });

    EventOk
}
