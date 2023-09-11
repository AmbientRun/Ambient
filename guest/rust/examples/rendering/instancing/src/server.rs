use std::f64::consts::TAU;

use ambient_api::{
    core::{
        app::components::main_scene,
        camera::concepts::{
            PerspectiveInfiniteReverseCamera, PerspectiveInfiniteReverseCameraOptional,
        },
        prefab::components::prefab_from_url,
        rendering::components::{cast_shadows, light_ambient, light_diffuse, sun},
        transform::components::{lookat_target, rotation, scale, translation},
    },
    glam::EulerRot,
    prelude::*,
};

use packages::this::{assets, components::instance_index};

#[main]
pub async fn main() {
    PerspectiveInfiniteReverseCamera {
        local_to_world: Mat4::IDENTITY,
        near: 0.1,
        projection: Mat4::IDENTITY,
        projection_view: Mat4::IDENTITY,
        active_camera: 0.0,
        inv_local_to_world: Mat4::IDENTITY,
        fovy: 1.0,
        aspect_ratio: 1.0,
        perspective_infinite_reverse: (),
        optional: PerspectiveInfiniteReverseCameraOptional {
            translation: Some(Vec3::ONE * 5.),
            main_scene: Some(()),
            aspect_ratio_from_window: Some(entity::resources()),
            ..default()
        },
    }
    .make()
    .with(lookat_target(), vec3(0., 0., 0.))
    .spawn();

    const INSTANCES: UVec3 = uvec3(15, 15, 15);

    for i in 0..INSTANCES.x {
        for j in 0..INSTANCES.y {
            for k in 0..INSTANCES.z {
                let x = i as f32 / INSTANCES.x as f32;
                let y = j as f32 / INSTANCES.y as f32;
                let z = k as f32 / INSTANCES.z as f32;

                let index = k * INSTANCES.z * j * INSTANCES.y * INSTANCES.y + i;

                let model = if index % 2 == 0 {
                    assets::url("Teapot.glb")
                } else {
                    assets::url("Monkey.glb")
                };

                Entity::new()
                    .with(instance_index(), uvec3(i, j, k))
                    .with(translation(), (vec3(x, y, z) - 0.5) * 7.0)
                    .with(rotation(), default())
                    .with(scale(), Vec3::ONE * 0.2)
                    .with(cast_shadows(), ())
                    .with(prefab_from_url(), model)
                    .spawn();
            }
        }
    }

    Entity::new()
        .with(sun(), 0.0)
        .with(rotation(), Quat::from_rotation_y(-1.))
        .with(main_scene(), ())
        .with(light_diffuse(), Vec3::ONE * 5.0)
        .with(light_ambient(), Vec3::ZERO)
        .spawn();

    query(instance_index()).each_frame(|items| {
        let t = game_time().as_secs_f64();
        for (id, index) in items {
            entity::set_component(
                id,
                rotation(),
                Quat::from_euler(
                    EulerRot::ZXY,
                    (t % TAU) as f32 + index.z as f32 * 0.5,
                    (t % TAU) as f32 + index.x as f32 * 0.5,
                    (t % TAU) as f32 + index.y as f32 * 0.5,
                ),
            );
        }
    });
}
