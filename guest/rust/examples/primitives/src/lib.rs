use kiwi_api::{
    components::core::{
        camera::{aspect_ratio_from_window, perspective_infinite_reverse},
        game_objects::player_camera,
        primitives::{cube, quad},
        rendering::color,
        transform::{lookat_center, scale, translation},
    },
    prelude::*,
};

#[main]
pub async fn main() -> EventResult {
    entity::game_object_base()
        .with_default(player_camera())
        .with(translation(), vec3(5.0, 5.0, 4.0))
        .with(lookat_center(), vec3(0., 0., 0.))
        .with(perspective_infinite_reverse(), ())
        .with(aspect_ratio_from_window(), ())
        .spawn();

    entity::game_object_base()
        .with_default(cube())
        .with(translation(), vec3(0., 0., 1.))
        .spawn();
    entity::game_object_base()
        .with_default(quad())
        .with(scale(), vec3(5., 5., 5.))
        .with(color(), vec4(1., 0., 0., 1.))
        .spawn();

    EventOk
}
