use kiwi_api::{
    components::core::{
        app::main_scene,
        camera::{aspect_ratio_from_window, perspective_infinite_reverse},
        game_objects::player_camera,
        rendering::color,
        transform::{
            local_to_world, lookat_center, mesh_to_local, mesh_to_world, scale, translation,
        },
        ui::text,
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

    entity::game_object_base()
        .with(text(), "Hello world".to_string())
        .with(color(), vec4(1., 1., 1., 1.))
        .with(translation(), vec3(0., 0., 0.01))
        .with(scale(), Vec3::ONE * 0.05)
        .with_default(local_to_world())
        .with_default(mesh_to_local())
        .with_default(mesh_to_world())
        .with_default(main_scene())
        .spawn();

    EventOk
}
