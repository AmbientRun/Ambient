use crate::packages::this::components::player_camera_ref;
use ambient_api::{
    core::{
        app::components::{main_scene, name},
        camera::{
            components::aspect_ratio_from_window,
            concepts::make_perspective_infinite_reverse_camera,
        },
        messages::Frame,
        player::components::{is_player, local_user_id, user_id},
        transform::components::{local_to_parent, translation},
    },
    entity::{add_child, mutate_component_with_default},
    prelude::*,
};
use packages::{
    this::{
        components::player_intermediate_rotation,
        messages::{Input, Jump},
    },
    unit_schema::components::head_ref,
};

#[main]
pub fn main() {
    let mut cursor_lock = input::CursorLockGuard::new();
    Frame::subscribe(move |_| {
        let (delta, input) = input::get_delta();
        if !cursor_lock.auto_unlock_on_escape(&input) {
            return;
        }

        let mut displace = Vec2::ZERO;
        if input.keys.contains(&KeyCode::W) {
            displace.x += 1.0;
        }
        if input.keys.contains(&KeyCode::S) {
            displace.x -= 1.0;
        }
        if input.keys.contains(&KeyCode::A) {
            displace.y -= 1.0;
        }
        if input.keys.contains(&KeyCode::D) {
            displace.y += 1.0;
        }
        let rot = mutate_component_with_default(
            player::get_local(),
            player_intermediate_rotation(),
            Vec2::ZERO,
            |rot| *rot += delta.mouse_position * 0.01,
        );

        if input.keys.contains(&KeyCode::Space) {
            Jump {}.send_server_reliable();
        }

        Input {
            run_direction: displace,
            body_yaw: rot.x,
            head_pitch: rot.y,
            running: input.keys.contains(&KeyCode::LShift),
            ducking: input.keys.contains(&KeyCode::LControl),
            shooting: input.mouse_buttons.contains(&MouseButton::Left),
        }
        .send_server_unreliable();
    });

    spawn_query((is_player(), user_id(), head_ref())).bind(move |players| {
        for (id, (_, user, head)) in players {
            let local_user_id =
                entity::get_component(entity::resources(), local_user_id()).unwrap();
            if user == local_user_id {
                eprintln!("Attaching camera to player {}", user);
                let camera = Entity::new()
                    .with_merge(make_perspective_infinite_reverse_camera())
                    .with(aspect_ratio_from_window(), EntityId::resources())
                    .with(main_scene(), ())
                    .with(translation(), -Vec3::Z * 4.)
                    .with(local_to_parent(), Default::default())
                    .with(name(), "Camera".to_string())
                    .spawn();
                add_child(head, camera);

                entity::add_components(id, Entity::new().with(player_camera_ref(), camera));
            }
        }
    });
}
