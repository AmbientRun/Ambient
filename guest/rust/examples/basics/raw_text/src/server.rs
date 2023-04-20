use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        rendering::color,
        text::text,
        transform::{
            local_to_world, lookat_target, mesh_to_local, mesh_to_world, scale, translation,
        },
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    prelude::*,
};

#[main]
pub fn main() {
    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with_default(main_scene())
        .with(translation(), vec3(5., 5., 4.))
        .with(lookat_target(), vec3(0., 0., 0.))
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with(text(), "Hello world".to_string())
        .with(color(), vec4(1., 1., 1., 1.))
        .with(translation(), vec3(0., 0., 0.01))
        .with(scale(), Vec3::ONE * 0.05)
        .with_default(local_to_world())
        .with_default(mesh_to_local())
        .with_default(mesh_to_world())
        .with_default(main_scene())
        .spawn();
}
