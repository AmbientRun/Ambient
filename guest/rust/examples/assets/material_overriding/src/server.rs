use ambient_api::{
    core::{
        app::components::main_scene,
        prefab::components::{prefab_from_url, spawned},
        primitives::components::quad,
        rendering::components::{cast_shadows, light_ambient, light_diffuse, sun},
        transform::components::{local_to_world, rotation, scale},
    },
    prelude::*,
};

use packages::{
    orbit_camera::concepts::{OrbitCamera, OrbitCameraOptional},
    this::{assets, components::is_the_best},
};

#[main]
pub async fn main() {
    OrbitCamera {
        is_orbit_camera: (),
        optional: OrbitCameraOptional {
            camera_angle: Some(vec2(135f32.to_radians(), 20f32.to_radians())),
            camera_distance: Some(3.),
            ..default()
        },
    }
    .spawn();

    Entity::new()
        .with(local_to_world(), Mat4::IDENTITY)
        .with(quad(), ())
        .with(scale(), Vec3::ONE * 2.0)
        .spawn();

    Entity::new()
        .with(local_to_world(), Mat4::IDENTITY)
        .with(sun(), 0.0)
        .with(rotation(), Quat::from_rotation_y(-1.))
        .with(main_scene(), ())
        .with(light_diffuse(), Vec3::ONE * 5.0)
        .with(light_ambient(), Vec3::ZERO)
        .spawn();

    let model = Entity::new()
        .with(local_to_world(), Mat4::IDENTITY)
        .with(cast_shadows(), ())
        .with(prefab_from_url(), assets::url("Teapot.glb"))
        .with(is_the_best(), true)
        .spawn();

    let _ = entity::wait_for_component(model, spawned()).await;

    println!("Entity components: {:?}", entity::get_all_components(model));
}
