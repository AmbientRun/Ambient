use ambient_api::{
    animation::{self, BindId},
    core::{
        model::components::model_loaded,
        player::components::is_player,
        prefab::components::prefab_from_url,
        // primitives::quad,
        rendering::components::color, //pbr_material_from_url
        transform::{
            components::{local_to_parent, reset_scale, rotation, scale, translation},
            concepts::make_transformable,
        },
    },
    prelude::*,
};

use embers::{afps_fpsmodel::assets, afps_schema::components};

#[main]
pub fn main() {
    spawn_query((is_player(), components::player_model_ref())).bind(move |results| {
        for (_, (_, model)) in results {
            run_async(async move {
                entity::wait_for_component(model, model_loaded()).await;
                println!("___model loaded___waiting for binding__");
                let hand = animation::get_bone_by_bind_id(model, &BindId::RightHand);
                if hand.is_none() {
                    return;
                }
                let hand = hand.unwrap();
                let gun = Entity::new()
                    .with_merge(make_transformable())
                    .with(prefab_from_url(), assets::url("red.glb"))
                    .with(translation(), vec3(-0.06, 0.2, 0.0))
                    .with(
                        rotation(),
                        Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)
                            * Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
                    )
                    .with(scale(), Vec3::ONE * 0.3)
                    .with(color(), vec4(1.0, 1.0, 0.0, 1.0))
                    .with(local_to_parent(), Default::default())
                    .with(reset_scale(), ())
                    .spawn();

                entity::add_child(hand, gun);
            });
        }
    });
}
