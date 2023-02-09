use elements_ecs::{EntityId, World};
use elements_input::{player_prev_raw_input, player_raw_input, PlayerRawInput};

pub fn get_raw_input(world: &World, player_id: EntityId) -> Option<PlayerRawInput> {
    world.get_cloned(player_id, player_raw_input()).ok()
}

pub fn get_prev_raw_input(world: &World, player_id: EntityId) -> Option<PlayerRawInput> {
    world.get_cloned(player_id, player_prev_raw_input()).ok()
}
