use ambient_api::{
    entity::{get_component, set_component},
    prelude::*,
};
use packages::{
    this::concepts::CharacterMovement,
    unit_schema::components::{
        is_on_ground, jumping, run_speed_multiplier, speed, strafe_speed_multiplier,
        vertical_velocity,
    },
};

const FALLING_VSPEED: f32 = 0.4;

#[main]
pub fn main() {
    query(CharacterMovement::as_query()).each_frame(move |list| {
        for (unit_id, data) in list {
            let scale_factor = if data.running {
                get_component(unit_id, run_speed_multiplier()).unwrap_or(1.5)
            } else {
                1.
            } * get_component(unit_id, speed()).unwrap_or(0.06);
            let speed = scale_factor
                * vec2(
                    get_component(unit_id, strafe_speed_multiplier()).unwrap_or(0.8),
                    1.,
                );
            let displace = data.rotation
                * (data.run_direction.normalize_or_zero() * speed).extend(data.vertical_velocity);
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
