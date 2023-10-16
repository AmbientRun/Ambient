use crate::packages::unit_schema::components::air_speed_multiplier;
use ambient_api::{
    entity::{add_component, get_component, set_component},
    prelude::*,
};
use packages::{
    this::concepts::CharacterMovement,
    unit_schema::components::{
        is_on_ground, jumping, run_speed_multiplier, speed, strafe_speed_multiplier,
        unit_displacement, vertical_velocity,
    },
};

const FALLING_VSPEED: f32 = 0.4;

fn get_speed(id: EntityId, data: &CharacterMovement) -> Vec2 {
    let speed = get_component(id, speed()).unwrap_or(0.06);
    let run_mul = if data.running {
        get_component(id, run_speed_multiplier()).unwrap_or(1.5)
    } else {
        1.
    };
    let air_mul = if !data.is_on_ground {
        get_component(id, air_speed_multiplier()).unwrap_or(0.1)
    } else {
        1.
    };
    let strafe_mul = get_component(id, strafe_speed_multiplier()).unwrap_or(0.8);
    speed * run_mul * air_mul * vec2(strafe_mul, 1.)
}

#[main]
pub fn main() {
    query(CharacterMovement::as_query()).each_frame(move |list| {
        for (unit_id, data) in list {
            let speed = get_speed(unit_id, &data);
            let mut displace =
                data.rotation * (data.run_direction.normalize_or_zero() * speed).extend(0.);
            displace.z = data.vertical_velocity;
            add_component(unit_id, unit_displacement(), displace);
            let collision = physics::move_character(unit_id, displace, 0.01, delta_time());
            entity::add_component(unit_id, is_on_ground(), collision.down);
            if collision.down {
                if data.vertical_velocity != 0. {
                    entity::set_component(unit_id, vertical_velocity(), 0.0);
                }
                if data.jumping {
                    set_component(unit_id, jumping(), false);
                }
            } else {
                entity::mutate_component(unit_id, vertical_velocity(), |vertical_velocity| {
                    *vertical_velocity -= FALLING_VSPEED * delta_time(); // 1/60 second for example
                });
            }
        }
    });
}
