use kiwi_api::{
    components::core::{
        game_objects::player_camera,
        object::object_from_url,
        player::player,
        primitives::quad,
        rendering::color,
        transform::{lookat_center, scale, translation},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    entity::{AnimationAction, AnimationController},
    player::KeyCode,
    prelude::*,
};

#[main]
pub async fn main() -> EventResult {
    make_perspective_infinite_reverse_camera()
        .with_default(player_camera())
        .with(translation(), vec3(2., 2., 3.0))
        .with(lookat_center(), vec3(0., 0., 1.))
        .spawn();

    make_transformable()
        .with_default(quad())
        .with(scale(), Vec3::ONE * 10.)
        .with(color(), vec4(0.5, 0.5, 0.5, 1.))
        .spawn();

    let unit_id = make_transformable()
        .with(
            object_from_url(),
            "assets/DeadMeetsLeadContent/Data/Models/Props/Altar1.glb".to_string(),
        )
        .spawn();

    EventOk
}
