use ambient_api::{core::transform::components::local_to_world, prelude::*};
use packages::{
    spawner_vehicle::messages::VehicleSpawn, tangent_schema::player::components as pc,
    this::messages::VehicleSpawnRequest,
};

#[main]
pub fn main() {
    VehicleSpawnRequest::subscribe(|ctx, msg| {
        let Some(player_id) = ctx.client_entity_id() else {
            return;
        };

        if entity::has_component(player_id, pc::vehicle_ref()) {
            return;
        }

        let Some(ltw) = entity::get_component(player_id, pc::character_ref())
            .and_then(|c| entity::get_component(c, local_to_world()))
        else {
            return;
        };

        let (_, rotation, translation) = ltw.to_scale_rotation_translation();
        VehicleSpawn {
            def_id: msg.def_id,
            position: translation,
            rotation: Some(rotation),
            driver_id: Some(player_id),
        }
        .send_local_broadcast(false);
    });
}
