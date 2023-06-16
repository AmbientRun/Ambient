use std::f64::consts::TAU;

use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        prefab::{prefab_from_url, spawned},
        transform::{lookat_target, rotation, translation},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    glam::EulerRot,
    prelude::*,
};

#[main]
pub async fn main() {
    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with_default(main_scene())
        .with(translation(), vec3(2., 2., 1.))
        .with(lookat_target(), vec3(0., 0., 0.))
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with_default(sun())
        .with(rotation(), Quat::from_rotation_y(-1.))
        .with_default(main_scene())
        .with(light_diffuse(), Vec3::ONE * 5.0)
        .with(light_ambient(), Vec3::ZERO)
        .spawn();

    let model = Entity::new()
        .with_merge(make_transformable())
        .with_default(cast_shadows())
        .with(prefab_from_url(), asset::url("assets/Teapot.glb").unwrap())
        .with(components::is_the_best(), true)
        .spawn();

    entity::wait_for_component(model, spawned()).await;

    ambient_api::messages::Frame::subscribe(move |_| {
        let t = time().as_secs_f64();
        entity::set_component(
            model,
            rotation(),
            Quat::from_euler(
                EulerRot::ZXY,
                (t % TAU) as f32,
                (t * 2.0).sin() as f32 * 0.5,
                0.0,
            ),
        );
    });
}
