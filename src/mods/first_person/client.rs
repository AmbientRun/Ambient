use ambient_api::{
    animation::{get_bone_by_bind_id, AnimationPlayer, BindId, BlendNode, PlayClipFromUrlNode},
    components::core::{
        animation::apply_animation_player, camera::aspect_ratio_from_window, model::model_loaded,
        prefab::prefab_from_url, primitives::quad, transform::reset_scale,
    },
    concepts::{make_perspective_infinite_reverse_camera, make_sphere, make_transformable},
    element::to_owned,
    entity::{add_child, add_component, wait_for_component},
    prelude::*,
};

#[main]
pub fn main() {
    // let cam = query(components::player_head_ref()).build();
    let shotcount = std::sync::atomic::AtomicUsize::new(0);
    // spawn_query(player()).bind(async |w| {
    //     wait_for_component(unit_id, model_loaded()).await;
    // });
    // let capoeira = PlayClipFromUrlNode::new(
    //     asset::url("assets/Capoeira.fbx/animations/mixamo.com.anim").unwrap(),
    // );
    // let robot = PlayClipFromUrlNode::new(
    //     asset::url("assets/Robot Hip Hop Dance.fbx/animations/mixamo.com.anim").unwrap(),
    // );
    // let blend = BlendNode::new(&capoeira, &robot, 0.);
    // let anim_player = AnimationPlayer::new(&blend);
    // add_component(unit_id, apply_animation_player(), anim_player.0);
    run_async(async move {
        let play_id = player::get_local();
        let model = entity::get_component(play_id, components::model_ref()).unwrap();
        wait_for_component(model, model_loaded()).await;
        println!("model loaded");
        let hand = get_bone_by_bind_id(model, &BindId::RightHand).unwrap();
        let ball = Entity::new()
            .with_merge(make_transformable())
            .with_merge(make_sphere())
            .with(scale(), vec3(0.3, 0.3, 0.3))
            .with(color(), vec4(0.0, 1.0, 0.0, 1.0))
            .with_default(local_to_parent())
            .with_default(reset_scale())
            .spawn();
        add_child(hand, ball);
    });

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
