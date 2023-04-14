use ambient_ecs::World;

use crate::shared::{conversion::IntoBindgen, wit};

pub fn get_by_user_id(
    world: &World,
    user_id: String,
) -> anyhow::Result<Option<wit::types::EntityId>> {
    Ok(ambient_core::player::get_by_user_id(world, &user_id).into_bindgen())
}
