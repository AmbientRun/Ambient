use std::f32::consts::PI;

use ambient_api::{
    core::transform::components::{scale, translation},
    prelude::*,
};

use packages::this::components::*;

#[main]
pub fn main() {
    basics();
    setup_tweens();
}

fn basics() {
    change_query(loop_length())
        .track_change(loop_length())
        .bind(|drops| {
            for (drop, looplen) in drops {
                if looplen == 0. {
                    entity::remove_component(drop, calculated_loop_speed());
                } else {
                    entity::add_component(drop, calculated_loop_speed(), 1. / looplen);
                }
            }
        });
    query(calculated_loop_speed()).each_frame(|drops| {
        let dt = delta_time();
        for (drop, loopspd) in drops {
            entity::mutate_component_with_default(drop, age(), 0., move |age| {
                let new_age = *age + loopspd * dt;
                if new_age > 1. {
                    entity::mutate_component_with_default(drop, loop_index(), 0, |li| {
                        *li = (*li + new_age.floor() as u8) % u8::MAX as u8
                    });
                    *age = new_age % 1.;
                } else {
                    *age = new_age;
                }
            });
        }
    });
}

fn setup_tweens() {
    change_query((age(), tween_atob_a(), tween_atob_b()))
        .track_change(age())
        .bind(|drops| {
            for (drop, (age, a, b)) in drops {
                entity::add_component(drop, translation(), a.lerp(b, age % 1.));
            }
        });
    change_query((age(), tween_sinescale()))
        .track_change(age())
        .bind(|drops| {
            for (drop, (t, maxscale)) in drops {
                entity::add_component(drop, scale(), Vec3::splat((t * PI).sin() * maxscale));
            }
        });
}

// fn remap32_wrap(value: f32, low1: f32, high1: f32, low2: f32, high2: f32) -> f32 {
//     (low2 + (value - low1) / (high1 - low1)) % 1. * (high2 - low2)
// }
