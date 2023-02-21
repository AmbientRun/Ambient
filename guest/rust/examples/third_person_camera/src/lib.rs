use ambient_api::{
    components::core::{
        game_objects::player_camera,
        player::{player, user_id},
        primitives::{cube, quad},
        rendering::color,
        transform::{lookat_center, scale, translation, rotation},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    prelude::*, player::KeyCode, rand,
};
use components::player_camera_ref;

#[main]
pub async fn main() -> EventResult {
    Entity::new()
        .with_merge(make_transformable())
        .with_default(quad())
        .with(scale(), Vec3::ONE * 10.)
        .with(color(), vec4(1., 0., 0., 1.))
        .spawn();

    spawn_query((player(), user_id())).bind(move |players| {
        for (id, (_, user)) in players {
            let camera = Entity::new()
                .with_merge(make_perspective_infinite_reverse_camera())
                .with_default(player_camera())
                .with(user_id(), user)
                .with(translation(), Vec3::ONE * 5.)
                .with(lookat_center(), vec3(0., 0., 0.))
                .spawn();
            entity::add_components(
                id,
                Entity::new()
                    .with_merge(make_transformable())
                    .with_default(cube())
                    .with(player_camera_ref(), camera)
                    .with(color(), rand::random()),
            );
        }
    });

    query((player(), player_camera_ref()))
        .build()
        .bind(move |players| {
            for (player_id, (_, camera_id)) in players {
                let Some((delta, pressed)) = player::get_raw_input_delta(player_id) else { continue; };

                let forward = entity::get_component(player_id, rotation()).unwrap() * Vec3::X;
                let right = entity::get_component(player_id, rotation()).unwrap() * Vec3::Y;
                let speed = 0.1;
                
                if pressed.keys.contains(&KeyCode::W) {
                    entity::mutate_component(player_id, translation(), |x| *x += forward * speed);
                }
                if pressed.keys.contains(&KeyCode::S) {
                    entity::mutate_component(player_id, translation(), |x| *x += -forward * speed);
                }
                if pressed.keys.contains(&KeyCode::A) {
                    entity::mutate_component(player_id, translation(), |x| *x += -right * speed);
                }
                if pressed.keys.contains(&KeyCode::D) {
                    entity::mutate_component(player_id, translation(), |x| *x += right * speed);
                }

                entity::mutate_component(player_id, rotation(), |x| *x *= Quat::from_rotation_z(delta.mouse_position.x * 0.01));

                let pos = entity::get_component(player_id, translation()).unwrap();
                entity::set_component(camera_id, lookat_center(), pos);
                entity::set_component(camera_id, translation(), pos - forward * 4. + Vec3::Z * 2.);
            }
        });

    EventOk
}
