use ambient_api::{
    animation::{self, AnimationPlayerRef, BindId, PlayClipFromUrlNodeRef},
    core::{
        animation::components::apply_animation_player,
        messages::Frame,
        model::components::model_loaded,
        prefab::components::prefab_from_url,
        transform::{components::rotation, concepts::Transformable},
    },
    entity::mutate_component,
    prelude::*,
};
use packages::this::assets;
#[main]
pub async fn main() {
    // Model
    let zombie = Entity::new()
        .with_merge(Transformable {
            local_to_world: Default::default(),
            optional: Default::default(),
        })
        .with(prefab_from_url(), assets::url("Zombie1.x"))
        .spawn();

    let idle = PlayClipFromUrlNodeRef::new(assets::url("Zombie1.x/animations/Run1.anim"));
    let anim_player = AnimationPlayerRef::new(idle);
    entity::add_component(zombie, apply_animation_player(), anim_player.0);

    // Joint3

    // let _ = entity::wait_for_component(zombie, model_loaded()).await;
    // let bone =
    //     animation::get_bone_by_bind_id(zombie, &BindId::Custom("Joint3".to_string())).unwrap();
    // Frame::subscribe(move |_| {
    //     mutate_component(bone, rotation(), |x| *x *= Quat::from_rotation_z(0.1));
    // });
}
