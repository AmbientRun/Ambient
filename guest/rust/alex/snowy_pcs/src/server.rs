use ambient_api::{core::player::components::is_player, prelude::*};
use packages::{
    character_animation::components::basic_character_animations,
    fps_controller::components::use_fps_controller,
    temperature::components::{temperature, temperature_src_radius, temperature_src_rate},
    this::components::pc_type_id,
};

const DEATH_TEMP: f32 = 21.13;
const NORMAL_TEMP: f32 = 36.65;

pub fn main() {
    spawn_query(is_player()).bind(|plrs| {
        for (plr, _) in plrs {
            entity::add_components(
                plr,
                Entity::new()
                    .with(use_fps_controller(), ())
                    .with(pc_type_id(), random::<u32>())
                    .with(basic_character_animations(), plr)
                    .with(temperature(), NORMAL_TEMP)
                    .with(temperature_src_rate(), 1.0)
                    .with(temperature_src_radius(), 8.0),
            );
        }
    });
}
