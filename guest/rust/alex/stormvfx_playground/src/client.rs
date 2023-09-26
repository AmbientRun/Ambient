use ambient_api::{
    core::{
        hierarchy::components::parent,
        primitives::concepts::Sphere,
        transform::{
            components::{local_to_parent, local_to_world, rotation, scale, translation},
            concepts::{Transformable, TransformableOptional},
        },
    },
    glam::EulerRot,
    prelude::*,
};

use packages::this::components::{dropage, dropagerate, dropvel};

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
    .spawn();
    for _ in 1..100 {
        let drop = Entity::new()
            .with_merge(Transformable::suggested().make())
            .with_merge(Sphere::suggested().make())
            .with(local_to_parent(), Mat4::IDENTITY)
            .with(parent(), stormparent)
            .with_merge(get_random_drop_start())
            .spawn();
        advance_drop(drop, None, random::<f32>() * 2.);
    }

    ambient_api::core::messages::Frame::subscribe(move |_| {
        entity::mutate_component(stormparent, rotation(), |rot| {
            let dt = delta_time();
            *rot = Quat::from_euler(EulerRot::XYZ, 0.02 * dt, 0.01 * dt, 0.05 * dt) * *rot;
        });
    });

    query((dropvel(), dropage(), dropagerate())).each_frame(|drops| {
        for (drop, (vel, age, agerate)) in drops {
            advance_drop(drop, Some((vel, age, agerate)), delta_time());
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
        entity::set_component(drop, scale(), Vec3::splat((age2 * 3.14).sin()));
    }
}

fn get_random_drop_start() -> Entity {
    Entity::new()
        .with(
            translation(),
            vec3(0., rand_range(-50., 50.), rand_range(-50., 50.)),
        )
        .with(dropage(), 0.)
        .with(dropagerate(), random::<f32>() * 0.5 + 0.5)
        .with(
            dropvel(),
            vec3(
                rand_range(-10., -40.),
                rand_range(-10., 10.),
                rand_range(-10., 10.),
            ),
        )
        .with(scale(), Vec3::ZERO)
}

fn rand_range(a: f32, b: f32) -> f32 {
    a + (b - a) * random::<f32>()
}
