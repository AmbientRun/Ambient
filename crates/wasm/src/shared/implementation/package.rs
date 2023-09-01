use ambient_ecs::World;
use ambient_network::ServerWorldExt;

use crate::shared::{conversion::IntoBindgen, wit};

pub fn get_entity_for_package_id(
    world: &World,
    package_id: String,
) -> anyhow::Result<Option<wit::types::EntityId>> {
    Ok(world
        .synced_resource(ambient_package_semantic_native::package_id_to_package_entity())
        .unwrap()
        .get(&package_id)
        .map(|id| (*id).into_bindgen()))
}
