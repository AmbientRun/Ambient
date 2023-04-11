use ambient_ecs::{query, EntityId, World};

pub use ambient_ecs::generated::components::core::player::{local_user_id, player, user_id};

pub fn get_player_by_user_id(world: &World, user_id: &str) -> Option<EntityId> {
    query(self::user_id()).incl(player()).iter(world, None).find_map(|(id, uid)| if uid == user_id { Some(id) } else { None })
}
