use kiwi_api::{
    components::core::{
        app::main_scene,
        game_objects::player_camera,
        rendering::color,
        transform::{
            local_to_world, lookat_center, mesh_to_local, mesh_to_world, scale, translation,
        },
        ui::text,
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    prelude::*,
};

#[main]
pub async fn main() -> EventResult {
    make_perspective_infinite_reverse_camera()
        .with_default(player_camera())
        .with(translation(), vec3(5., 5., 4.))
        .with(lookat_center(), vec3(0., 0., 0.))
        .spawn();

    make_transformable()
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
