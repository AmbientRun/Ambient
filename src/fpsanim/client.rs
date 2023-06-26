use ambient_api::{
    animation::{get_bone_by_bind_id, BindId},
    components::core::{model::model_loaded, prefab::prefab_from_url, transform::reset_scale},
    concepts::make_transformable,
    entity::{add_child, wait_for_component},
    prelude::*,
};

#[main]
pub fn main() {
    // let cam = query(components::player_head_ref()).build();
    let shotcount = std::sync::atomic::AtomicUsize::new(0);

    run_async(async move {
        loop {
            sleep(0.1).await;
            let play_id = player::get_local();
            let model = entity::get_component(play_id, components::model_ref());
            if model.is_none() {
                continue;
            }
            let model = model.unwrap();
            wait_for_component(model, model_loaded()).await;
            println!("[[[model loaded]]]");
            let hand = get_bone_by_bind_id(model, &BindId::RightHand).unwrap();
            let ball = Entity::new()
                .with_merge(make_transformable())
                // .with_merge(make_sphere())
                .with(
                    prefab_from_url(),
                    asset::url("assets/gun/m4a1_carbine.glb").unwrap(),
                )
                // y => far from body,
                .with(translation(), vec3(0.0, 0.2, 0.0))
                .with(
                    rotation(),
                    Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2),
                )
                // .with(scale(), Vec3::ONE)
                .with(scale(), Vec3::ONE * 0.01)
                .with(color(), vec4(1.0, 1.0, 0.0, 1.0))
                .with_default(local_to_parent())
                .with_default(reset_scale())
                .spawn();
            add_child(hand, ball);
            break;
        }
    });

    messages::FireSound::subscribe(|_, msg| {
        // println!("FireSound");
        let emitter = msg.source;
        spatial_audio::set_emitter(emitter);
        // remember: we change the rotation z on player_entity
        // entity::get_component(player::get_local(), components::player_head_ref()).unwrap(),
        spatial_audio::set_listener(player::get_local());
        spatial_audio::play_sound_on_entity(asset::url("assets/sound/m4a1.ogg").unwrap(), emitter);
    });

    // TODO: this cannot be put into the Zombie mod?
    messages::HitZombie::subscribe(move |_source, msg| {
        let zb = msg.id;
        // let cam =
        //     entity::get_component(player::get_local(), components::player_head_ref()).unwrap();
        // println!("Hit zombie {:?}!", zb);
        spatial_audio::set_emitter(zb);
        spatial_audio::set_listener(player::get_local());
        spatial_audio::play_sound_on_entity(asset::url("assets/sound/hit.ogg").unwrap(), zb);
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
            if shotcount.load(std::sync::atomic::Ordering::SeqCst) % 60 == 0 {
                shoot = true;
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
        }

        // we need the continues shoot info here
        let left_pressed = input.mouse_buttons.contains(&MouseButton::Left);
        messages::Input::new(left_pressed, displace, input.mouse_delta).send_server_unreliable();
    });
}
