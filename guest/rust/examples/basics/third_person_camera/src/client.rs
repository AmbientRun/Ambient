use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        player::{local_user_id, player, user_id},
        transform::{lookat_center, rotation, translation},
    },
    concepts::make_perspective_infinite_reverse_camera,
    message::client::{MessageExt, Target},
    player::KeyCode,
    prelude::*,
};
use components::player_camera_ref;

#[main]
fn main() {
    spawn_query((player(), user_id())).bind(move |players| {
        for (id, (_, user)) in players {
            if user == entity::get_component(entity::resources(), local_user_id()).unwrap() {
                let camera = Entity::new()
                    .with_merge(make_perspective_infinite_reverse_camera())
                    .with(aspect_ratio_from_window(), EntityId::resources())
                    .with_default(main_scene())
                    .with(user_id(), user)
                    .with(translation(), Vec3::ONE * 5.)
                    .with(lookat_center(), vec3(0., 0., 0.))
                    .spawn();

                entity::add_components(id, Entity::new().with(player_camera_ref(), camera));
            }
        }
    });
    // Since we're only attaching player_camera_ref to the local player, this system will only
    // run for the local player
    query((player(), player_camera_ref(), translation(), rotation())).each_frame(move |players| {
        for (_, (_, camera_id, pos, rot)) in players {
            let forward = rot * Vec3::X;
            entity::set_component(camera_id, lookat_center(), pos);
            entity::set_component(camera_id, translation(), pos - forward * 4. + Vec3::Z * 2.);
        }
    });

    on(event::FRAME, |_| {
        let (delta, pressed) = player::get_raw_input_delta();

        let mut displace = Vec2::ZERO;
        if pressed.keys.contains(&KeyCode::W) {
            displace.x += 1.0;
        }
        if pressed.keys.contains(&KeyCode::S) {
            displace.x -= 1.0;
        }
        if pressed.keys.contains(&KeyCode::A) {
            displace.y -= 1.0;
        }
        if pressed.keys.contains(&KeyCode::D) {
            displace.y += 1.0;
        }

        messages::Input::new(displace, delta.mouse_position.x).send(Target::RemoteReliable);
    });
}
