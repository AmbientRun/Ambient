use elements_ecs::{EntityId, World};
use tilt_runtime_player::{prev_raw_input, raw_input, RawInput};

pub fn get_raw_input(world: &World, player_id: EntityId) -> Option<RawInput> {
    world.get_cloned(player_id, raw_input()).ok()
}

pub fn get_prev_raw_input(world: &World, player_id: EntityId) -> Option<RawInput> {
    world.get_cloned(player_id, prev_raw_input()).ok()
}
