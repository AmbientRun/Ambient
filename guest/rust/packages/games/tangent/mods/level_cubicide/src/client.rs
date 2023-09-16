use ambient_api::{
    core::{
        rendering::components::{fog_density, sun},
        transform::components::translation,
    },
    prelude::*,
};

use packages::tangent_schema::components::player_vehicle;

mod shared;

#[main]
fn main() {
    // Automatically adjust the density of the fog on a cycle
    query(sun()).each_frame(move |suns| {
        const BASE: f32 = 0.02;
        const AMPLITUDE: f32 = 0.06;
        // How many metres the player can travel before the fog is at its maximum density
        const TRANSITION_LENGTH: f32 = 4.0;

        let Some(local_translation) = entity::get_component(player::get_local(), player_vehicle())
            .and_then(|pv| entity::get_component(pv, translation()))
        else {
            return;
        };

        let sdf = shared::level(local_translation.xy());

        for (sun_id, _) in suns {
            // If the player is in the level carve-out, the fog should be light.
            // Otherwise, it should be heavy.
            let new_density = BASE + AMPLITUDE * (sdf / TRANSITION_LENGTH).clamp(0.0, 1.0);
            entity::set_component(sun_id, fog_density(), new_density)
        }
    });
}
