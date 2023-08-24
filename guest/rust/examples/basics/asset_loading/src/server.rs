use std::f32::consts::TAU;

use ambient_api::{
    core::{
        app::components::main_scene,
        camera::{
            components::aspect_ratio_from_window,
            concepts::make_perspective_infinite_reverse_camera,
        },
        messages::Frame,
        prefab::components::{prefab_from_url, spawned},
        primitives::components::quad,
        rendering::components::{cast_shadows, light_ambient, light_diffuse, sun},
        transform::{
            components::{lookat_target, rotation, scale, translation},
            concepts::make_transformable,
        },
    },
    glam::EulerRot,
    prelude::*,
};

use packages::this::{assets, components::is_the_best};

#[main]
pub async fn main() {
    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with(main_scene(), ())
        .with(translation(), vec3(2., 2., 1.))
        .with(lookat_target(), vec3(0., 0., 0.))
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with(quad(), ())
        .with(scale(), Vec3::ONE * 2.0)
        .spawn();

    println!("Hello, Ambient!");

    Entity::new()
        .with_merge(make_transformable())
        .with(sun(), 0.0)
        .with(rotation(), Quat::from_rotation_y(-1.))
        .with(main_scene(), ())
        .with(light_diffuse(), Vec3::ONE * 5.0)
        .with(light_ambient(), Vec3::ZERO)
        .spawn();

    let model = Entity::new()
        .with_merge(make_transformable())
        .with(cast_shadows(), ())
        .with(prefab_from_url(), assets::url("Teapot.glb"))
        .with(is_the_best(), true)
        .spawn();

    entity::wait_for_component(model, spawned()).await;

    println!("Entity components: {:?}", entity::get_all_components(model));

    Frame::subscribe(move |_| {
        let t = game_time().as_secs_f32();
        entity::set_component(
            model,
            rotation(),
            Quat::from_euler(EulerRot::ZXY, t % TAU, (t * 2.0).sin() * 0.5, 0.0),
        );
    });
}
