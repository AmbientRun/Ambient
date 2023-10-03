use ambient_api::{core::player::components::is_player, prelude::*};
use packages::this::components::player_count;

#[main]
pub fn main() {
    query(is_player()).each_frame(|plrs| {
        entity::add_component(packages::this::entity(), player_count(), plrs.len() as u8);
    });
}
