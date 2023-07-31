use ambient_api::{
    components::core::player::player,
    entity::{
        add_component, add_component_if_required, mutate_component, remove_component, set_component,
    },
    prelude::*,
};

#[main]
pub fn main() {
    spawn_query(player()).bind(|results| {
        for (id, _) in results {
            add_component_if_required(id, components::player_last_frame(), 0);
        }
    });

    let server_entity_id = Entity::new().spawn();
    add_component(server_entity_id, components::server_frame(), 0);

    ambient_api::messages::Frame::subscribe(move |_| {
        mutate_component(server_entity_id, components::server_frame(), |frame| {
            *frame += 1
        });
    });

    messages::FrameSeen::subscribe(move |source, msg| {
        let Some(player_entity_id) = source.client_entity_id() else {
            eprintln!("Received message from unknown client");
            return;
        };
        set_component(player_entity_id, components::player_last_frame(), msg.frame);
    });
}
