use ambient_api::{
    animation::{get_bone_by_bind_id, BindId},
    core::{
        ecs::components::{children, parent},
        model::components::model_loaded,
        player::components::player,
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

use afps_gun::messages;
#[main]
pub fn main() {
    messages::Switch::subscribe(|source, msg| {
        println!("Switch message received: {:?}", msg);
        let player_id = msg.player_id;
        let gun_type = msg.gun_type;
        let model =
            entity::get_component(player_id, afps_schema::components::player_model_ref()).unwrap();
        let hand = get_bone_by_bind_id(model, &BindId::RightHand);
        if hand.is_none() {
            println!("no hand found");
            return;
        }
        let hand = hand.unwrap();
        let gun = Entity::new()
            .with_merge(make_transformable())
            .with(prefab_from_url(), {
                if gun_type == 0 {
                    asset::url("afps_fpsmodel/assets/red.glb").unwrap()
                } else if gun_type == 1 {
                    asset::url("afps_fpsmodel/assets/green.glb").unwrap()
                } else {
                    asset::url("afps_fpsmodel/assets/blue.glb").unwrap()
                }
            })
            .with(translation(), vec3(-0.06, 0.2, 0.0))
            .with(
                rotation(),
                Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)
                    * Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
            )
            .with(scale(), Vec3::ONE * 0.3)
            .with_default(local_to_parent())
            .with_default(reset_scale())
            .spawn();

        entity::mutate_component(hand, children(), |v| v.clear());
        entity::add_child(hand, gun);
    });
}
