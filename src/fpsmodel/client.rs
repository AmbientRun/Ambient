use ambient_api::{
    animation::{get_bone_by_bind_id, BindId},
    components::core::{model::model_loaded, prefab::prefab_from_url, transform::reset_scale},
    concepts::make_transformable,
    prelude::*,
};

use ambient_api::components::core::{
    player::player,
    primitives::{cube, quad},
    rendering::{color, pbr_material_from_url},
    transform::{local_to_parent, rotation, scale, translation},
};

#[main]
pub fn main() {
    spawn_query((player(), components::player_model_ref())).bind(move |results| {
        for (_, (_, model)) in results {
            run_async(async move {
                entity::wait_for_component(model, model_loaded()).await;
                println!("___model loaded___");
                let hand = get_bone_by_bind_id(model, &BindId::RightHand).unwrap();
                let gun = Entity::new()
                    .with_merge(make_transformable())
                    .with(
                        prefab_from_url(),
                        asset::url("assets/gun/m4a1_carbine.glb").unwrap(),
                    )
                    // y => far from body; need more tuning
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
                let f = Entity::new()
                    .with_merge(make_transformable())
                    .with_default(quad())
                    .with(scale(), Vec3::ONE * 100.)
                    .with(
                        pbr_material_from_url(),
                        asset::url("assets/pipeline.toml/5/mat.json").unwrap(),
                    )
                    .with_default(local_to_parent())
                    .spawn();
                // let c = Entity::new()
                //     .with_merge(make_transformable())
                //     .with_default(cube())
                //     .with(scale(), Vec3::ONE * 200.1)
                //     .with(color(), vec4(1.0, 0.0, 0.0, 1.0))
                //     .with(
                //         pbr_material_from_url(),
                //         asset::url("assets/pipeline.toml/0/mat.json").unwrap(),
                //     )
                //     .with_default(local_to_parent())
                //     .spawn();
                entity::add_child(hand, gun);
                entity::add_child(hand, f);
                // entity::add_child(hand, c);
            });
        }
    });
}
