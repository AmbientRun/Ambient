use ambient_api::{
    core::{
        app::components::main_scene,
        camera::{
            components::aspect_ratio_from_window, concepts::make_PerspectiveInfiniteReverseCamera,
        },
        messages::Frame,
        player::components::{is_player, local_user_id, user_id},
        transform::components::{lookat_target, rotation, translation},
    },
    prelude::*,
};

use packages::this::{
    components::{camera_follow_distance, player_camera_ref},
    messages::Input,
};

#[main]
fn main() {
    eprintln!("Client started");
    spawn_query((is_player(), user_id())).bind(move |players| {
        for (id, (_, user)) in players {
            let local_user_id =
                entity::get_component(entity::resources(), local_user_id()).unwrap();
            eprintln!("Player joined {user}\nlocal_user_id: {local_user_id:?}");
            // First, we check if this player is the "local" player, and only then do we attach a camera
            if user == local_user_id {
                eprintln!("Attaching camera to player {}", user);
                let camera = Entity::new()
                    .with_merge(make_PerspectiveInfiniteReverseCamera())
                    .with(aspect_ratio_from_window(), EntityId::resources())
                    .with(main_scene(), ())
                    .with(user_id(), user)
                    .with(translation(), Vec3::ONE * 5.)
                    .with(lookat_target(), vec3(0., 0., 0.))
                    .spawn();

                entity::add_components(id, Entity::new().with(player_camera_ref(), camera));
            }
        }
    });
    // Since we're only attaching player_camera_ref to the local player, this system will only
    // run for the local player
    query((
        is_player(),
        player_camera_ref(),
        translation(),
        rotation(),
        camera_follow_distance(),
    ))
    .each_frame(move |players| {
        for (_, (_, camera_id, pos, rot, dist)) in players {
            entity::set_component(camera_id, lookat_target(), pos);
            let offset = rot * vec3(-1.0, 0.0, 0.2).normalize() * dist;
            entity::set_component(camera_id, translation(), pos + offset);
        }
    });

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

        Input::new(displace, delta.mouse_position.x, input.mouse_wheel).send_server_reliable();
    });
}
