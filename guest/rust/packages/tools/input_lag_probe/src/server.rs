use ambient_api::{core::player::components::is_player, prelude::*};
use packages::this::{
    components::{input_lag, input_timestamp},
    messages::Input,
};

#[main]
pub fn main() {
    spawn_query(is_player()).bind(|players| {
        for (id, _) in players {
            entity::add_components(
                id,
                Entity::new()
                    .with(input_timestamp(), Duration::ZERO)
                    .with(input_lag(), Duration::ZERO),
            );
        }
    });
    Input::subscribe(|ctx, Input { timestamp, lag }| {
        if let Some(id) = ctx.client_entity_id() {
            entity::set_component(id, input_timestamp(), timestamp);
            entity::set_component(id, input_lag(), lag);
        }
    });
}
