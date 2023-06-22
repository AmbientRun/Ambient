use ambient_api::prelude::*;

#[main]
fn main() {
    // let cam = query(components::player_head_ref()).build();
    let shotcount = std::sync::atomic::AtomicUsize::new(0);

    ambient_api::messages::Frame::subscribe(move |_| {
        let (_delta, input) = input::get_delta();

        let mut displace = Vec2::ZERO;
        if input.keys.contains(&KeyCode::W) {
            displace.y -= 1.0;
        }
        if input.keys.contains(&KeyCode::S) {
            displace.y += 1.0;
        }
        if input.keys.contains(&KeyCode::A) {
            displace.x -= 1.0;
        }
        if input.keys.contains(&KeyCode::D) {
            displace.x += 1.0;
        }

        let mut shoot = false;
        if input.mouse_buttons.contains(&MouseButton::Left) {
            if shotcount.load(std::sync::atomic::Ordering::SeqCst) % 5 == 0 {
                shoot = true;
                // gunsound.volume(0.6).play();
            }
        }
        shotcount.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let player_id = player::get_local();

        if shoot {
            // println!("shoot");
            let cam = entity::get_component(player_id, components::player_head_ref()).unwrap();
            let window_size =
                entity::get_component(entity::resources(), window_logical_size()).unwrap();
            let ray = camera::screen_position_to_world_ray(
                cam,
                vec2(window_size.x as f32 / 2., window_size.y as f32 / 2.),
            );
            messages::Ray {
                ray_origin: ray.origin,
                ray_dir: ray.dir,
                source: player_id,
                type_action: 0,
            }
            .send_server_unreliable();

            // let head = cam;
            // spatial_audio::set_listener(cam);
            // spatial_audio::set_emitter(cam);
            // let url = asset::url("assets/sound/m4a1.wav").unwrap();
            // spatial_audio::play_sound_on_entity(url, cam);

            // let pitch = entity::mutate_component(player_id, components::player_pitch(), |pitch| {
            //     let recoil = random::<f32>() * 0.01;
            //     // println!("random::<f32>() * 0.01 {}", back);
            //     *pitch = *pitch - recoil;
            // })
            // .unwrap_or_default();

            // if let Some(cam) = entity::get_component(player_id, player_head_ref()) {
            //     entity::set_component(cam, rotation(), Quat::from_rotation_x(FRAC_PI_2+pitch));
            // }
        }

        messages::Input::new(displace, input.mouse_delta).send_server_unreliable();
    });

    // let mut cursor_lock = input::CursorLockGuard::new(true);
    // ambient_api::messages::Frame::subscribe(move |_| {
    //     let input = input::get();
    //     // if !cursor_lock.auto_unlock_on_escape(&input) {
    //     //     return;
    //     // }

    //     let mut displace = Vec2::ZERO;
    //     if input.keys.contains(&KeyCode::W) {
    //         displace.y -= 1.0;
    //     }
    //     if input.keys.contains(&KeyCode::S) {
    //         displace.y += 1.0;
    //     }
    //     if input.keys.contains(&KeyCode::A) {
    //         displace.x -= 1.0;
    //     }
    //     if input.keys.contains(&KeyCode::D) {
    //         displace.x += 1.0;
    //     }

    //     messages::Input::new(displace, input.mouse_delta).send_server_unreliable();
    // });
}
