use ambient_api::{
    core::{messages::Frame, player::components::is_player},
    entity::{add_component, add_component_if_required, mutate_component, set_component},
    prelude::*,
};
use packages::{
    afps_schema::components::player_last_frame,
    this::{components::server_frame, messages::FrameSeen},
};

#[main]
pub fn main() {
    spawn_query(is_player()).bind(|results| {
        for (id, _) in results {
            add_component_if_required(id, player_last_frame(), 0);
        }
    });

    let server_entity_id = Entity::new().spawn();
    add_component(server_entity_id, server_frame(), 0);

    Frame::subscribe(move |_| {
        mutate_component(server_entity_id, server_frame(), |frame| *frame += 1);
    });

    FrameSeen::subscribe(move |ctx, msg| {
        let Some(player_entity_id) = ctx.client_entity_id() else {
            eprintln!("Received message from unknown client");
            return;
        };
        set_component(player_entity_id, player_last_frame(), msg.frame);
    });
}
