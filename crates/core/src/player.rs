use ambient_ecs::{query, EntityId, World};

pub use ambient_ecs::generated::components::core::player::{local_user_id, player, user_id};

/// Returns the player entity for the given user ID, if it exists
pub fn get_by_user_id(world: &World, user_id: &str) -> Option<EntityId> {
    // TODO: Consider a more efficient implementation that caches the players in a HashMap or similar
    // O(N) might get a bit finicky with large numbers of players
    query(self::user_id())
        .incl(player())
        .iter(world, None)
        .find_map(|(id, uid)| if uid == user_id { Some(id) } else { None })
}
