use ambient_api::{
    animation::{AnimationPlayerRef, PlayClipFromUrlNodeRef},
    core::{
        animation::components::apply_animation_player, prefab::components::prefab_from_url,
        transform::concepts::Transformable,
    },
    prelude::*,
};
use packages::this::assets;

pub mod packages;

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
}
