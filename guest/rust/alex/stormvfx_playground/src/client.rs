use ambient_api::{
    core::{
        hierarchy::components::parent,
        model::components::model_from_url,
        primitives::concepts::Sphere,
        rendering::components::cast_shadows,
        transform::{
            components::{local_to_parent, local_to_world, rotation, scale, translation},
            concepts::{Transformable, TransformableOptional},
        },
    },
    glam::EulerRot,
    prelude::*,
};

use packages::this::components::{dropage, dropagerate, dropbill, dropvel};

#[main]
pub fn main() {
    let stormparent = Transformable {
        optional: TransformableOptional {
            translation: Some(Vec3::ZERO),
            rotation: Some(Quat::IDENTITY),
            scale: Some(Vec3::ONE),
        },
        ..Transformable::suggested()
    }
    .make()
    .with(local_to_world(), Mat4::IDENTITY)
    .with(scale(), Vec3::splat(0.02))
    // .with(rotation(), Quat::from_rotation_x(1.67)) // tilt on its side
    .spawn();

    // if let Some(camera) = camera::get_active(None) {
    //     if let Some(camera_pos) = entity::get_component(camera, translation()) {
    //         println!("GOT CAMERA POS, STORM'S LOOKIN' RIGHT AT YA :)");
    //         entity::add_component(stormparent, lookat_target(), camera_pos);
    //     } else {
    //         println!("/!\\ CAMERA FOUND BUT NO POSITION, STORM HAS NO ROTATION LOOKAT");
    //     }
    // } else {
    //     println!("/!\\ NO CAMERA FOUND, STORM HAS NO ROTATION LOOKAT");
    // }

    for _ in 1..400 {
        let drop = Entity::new()
            .with_merge(Transformable::suggested().make())
            .with(
                model_from_url(),
                packages::this::assets::url("fuzzy_circle_plane.glb"),
            )
            // .with_merge(Sphere::suggested().make())
            .with(local_to_parent(), Mat4::IDENTITY)
            .with(parent(), stormparent)
            .with(dropbill(), Quat::IDENTITY)
            .with_merge(get_random_drop_start())
            .spawn();
        advance_drop(drop, None, random::<f32>() * 2.);
    }

    // ambient_api::core::messages::Frame::subscribe(move |_| {
    //     entity::mutate_component(stormparent, rotation(), |rot| {
    //         let dt = delta_time();
    //         *rot = Quat::from_euler(EulerRot::XYZ, 0.02 * dt, 0.01 * dt, 0.05 * dt) * *rot;
    //     });
    // });

    query((dropvel(), dropage(), dropagerate())).each_frame(|drops| {
        for (drop, (vel, age, agerate)) in drops {
            advance_drop(drop, Some((vel, age, agerate)), delta_time());
        }
    });
    query(dropbill()).each_frame(|drops| {
        if let Some(camera) = camera::get_active(None) {
            if let Some(camera_inv_view) = entity::get_component(camera, local_to_world()) {
                let (_, camera_rotation, _) = camera_inv_view.to_scale_rotation_translation();
                for (drop, billboard_base_rotation) in drops {
                    entity::add_component(
                        drop,
                        rotation(),
                        camera_rotation * billboard_base_rotation,
                    );
                }
            }
        }
    });

    spawn_query(cast_shadows())
        .requires(dropbill())
        .bind(|drops| {
            for (drop, _) in drops {
                entity::remove_component(drop, cast_shadows());
            }
        });
}

fn advance_drop(drop: EntityId, params: Option<(Vec3, f32, f32)>, delta: f32) {
    let (vel, age, agerate) = params.unwrap_or_else(|| {
        (
            entity::get_component(drop, dropvel()).unwrap(),
            entity::get_component(drop, dropage()).unwrap(),
            entity::get_component(drop, dropagerate()).unwrap(),
        )
    });
    let age2 = age + agerate * delta;
    if age2 >= 1.0 {
        entity::set_component(drop, dropage(), age2 % 1.0);
        entity::set_components(drop, get_random_drop_start());
    } else {
        entity::set_component(drop, dropage(), age2);
        entity::mutate_component(drop, translation(), |pos| *pos = *pos + vel * delta);
        entity::set_component(drop, scale(), 3.0 * Vec3::splat((age2 * 3.14).sin()));
    }
}

fn get_random_drop_start() -> Entity {
    let startpos = vec3(rand_range(-50., 50.), rand_range(-50., 50.), -50.);
    let startvel = vec3(
        rand_range(-10., 10.),
        rand_range(-10., 10.),
        rand_range(20., 80.),
    );
    let agerate = random::<f32>() * 0.5 + 0.5;
    let lifetime = 1. / agerate;
    Entity::new()
        .with(
            translation(),
            startpos + startvel * random::<f32>() / lifetime,
        )
        .with(dropage(), 0.)
        .with(dropagerate(), agerate)
        .with(dropvel(), startvel)
        .with(scale(), Vec3::ZERO)
}

fn rand_range(a: f32, b: f32) -> f32 {
    a + (b - a) * random::<f32>()
}
