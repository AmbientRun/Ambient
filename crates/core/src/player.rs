use ambient_ecs::{components, query, Description, EntityId, Name, Networked, Resource, Store, World};

components!("player", {
    @[
        Networked, Store,
        Name["Player"],
        Description["This entity is a player.\nNote that this is a logical construct; a player's body may be separate from the player itself."]
    ]
    player: (),
    @[
        Networked, Store,
        Name["User ID"],
        Description["An identifier attached to all things owned by a user, and supplied by the user.\nThis can be attached to more than just the player; by convention, it is also attached to related entities, including their camera and body."]
    ]
    user_id: String,
    @[
        Networked, Store, Resource,
        Name["Local user ID"],
        Description["The user ID of the local player."]
    ]
    local_user_id: String,
});

pub fn get_player_by_user_id(world: &World, user_id: &str) -> Option<EntityId> {
    query(self::user_id()).incl(player()).iter(world, None).find_map(|(id, uid)| if uid == user_id { Some(id) } else { None })
}
