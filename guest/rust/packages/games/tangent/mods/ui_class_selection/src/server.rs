use ambient_api::prelude::*;
use packages::{
    tangent_schema::{concepts::PlayerClass, player::components::class},
    this::messages::ClassSetRequest,
};

#[main]
fn main() {
    ClassSetRequest::subscribe(|ctx, msg| {
        let Some(player_id) = ctx.client_entity_id() else {
            return;
        };
        if !PlayerClass::contained_by_spawned(msg.class_id) {
            return;
        }

        entity::add_component(player_id, class(), msg.class_id);
    });
}
