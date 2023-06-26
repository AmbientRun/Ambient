use ambient_api::{
    animation::{get_bone_by_bind_id, BindId},
    components::core::{
        camera::aspect_ratio_from_window, model::model_loaded, prefab::prefab_from_url,
        transform::reset_scale,
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    entity::{add_child, wait_for_component},
    prelude::*,
};

#[main]
pub fn main() {
    spawn_query((player(), components::player_model_ref())).bind(move |results| {
        for (_, (_, model)) in results {
            run_async(async move {
                // loop {
                // sleep(0.1).await;
                // let play_id = player::get_local();
                // let model = entity::get_component(play_id, components::player_model_ref());
                // if model.is_none() {
                //     continue;
                // }
                // let model = model.unwrap();
                wait_for_component(model, model_loaded()).await;
                println!("___model loaded___");
                let hand = get_bone_by_bind_id(model, &BindId::RightHand).unwrap();
                let gun = Entity::new()
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
                    .with(scale(), Vec3::ONE * 0.01)
                    .with(color(), vec4(1.0, 1.0, 0.0, 1.0))
                    .with_default(local_to_parent())
                    .with_default(reset_scale())
                    .spawn();
                add_child(hand, gun);
                // break;
                // }
            });
        }
    });
}
