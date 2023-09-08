use ambient_api::{
    core::{model::components::model_from_url, player::components::is_player},
    entity::add_components,
    prelude::*,
};
use packages::{base_assets, character_animation::components::basic_character_animations};

#[main]
fn main() {
    spawn_query(is_player()).bind(move |players| {
        for (id, _) in players {
            // Only attach models to other players
            if id != player::get_local() {
                add_components(
                    id,
                    Entity::new()
                        .with(model_from_url(), base_assets::assets::url("Y Bot.fbx"))
                        .with(basic_character_animations(), id),
                );
            }
        }
    });
}
