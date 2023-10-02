use ambient_api::{
    core::{player::components::is_player, transform::components::translation},
    prelude::*,
};
use packages::{
    character_animation::components::basic_character_animations,
    fps_controller::components::use_fps_controller,
    temperature::components::{temperature, temperature_src_radius, temperature_src_rate},
    this::components::pc_type_id,
};

#[main]
pub fn main() {
    // SHOULD BE GIVEN BY TEMPERATURE SCHEMA
    const TOO_HIGH_TEMP: f32 = 69.00;
    const DEATH_TEMP: f32 = 21.13;
    const NORMAL_TEMP: f32 = 36.65;
    const OVERNORMAL_TEMP: f32 = 37.20;
    const OVERNORMAL_COOLING_RATE: f32 = 20.00;

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

    query(translation())
        .requires(is_player())
        .each_frame(|plrs| {
            for (plr, pos) in plrs {
                if pos.z < -1.0 {
                    entity::mutate_component(plr, translation(), |pos| pos.z = 1.0);
                }
            }
        });

    query(temperature())
        .requires(is_player())
        .each_frame(|plrs| {
            for (plr, temp) in plrs {
                if temp < DEATH_TEMP || temp > TOO_HIGH_TEMP {
                    // todo: animate death
                    entity::add_component(plr, translation(), vec3(0., 0., 0.));
                    entity::set_component(plr, temperature(), NORMAL_TEMP);
                }
                fn remap32(value: f32, low1: f32, high1: f32, low2: f32, high2: f32) -> f32 {
                    low2 + (value - low1) * (high2 - low2) / (high1 - low1)
                }
                if temp > NORMAL_TEMP {
                    let cooling_rate = remap32(temp, NORMAL_TEMP, OVERNORMAL_TEMP, 0.0, 1.0)
                        .clamp(0.0, 1.0)
                        * OVERNORMAL_COOLING_RATE;
                    entity::mutate_component(plr, temperature(), |temp| {
                        *temp -= cooling_rate * delta_time()
                    });
                }
            }
        });
}
