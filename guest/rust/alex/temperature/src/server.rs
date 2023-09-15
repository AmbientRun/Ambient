use ambient_api::{core::transform::components::translation, prelude::*};
use packages::this::components::{
    temperature, temperature_src_falloff, temperature_src_radius, temperature_src_rate,
};

const DEFAULT_TEMPERATURE_SRC_FALLOFF: f32 = 0.5;

#[main]
pub fn main() {
    let find_temp_sources = query((
        temperature_src_rate(),
        temperature_src_radius(),
        translation(),
    ))
    .build();
    query((temperature(), translation())).each_frame(move |warm_bodies| {
        let temperature_sources = find_temp_sources.evaluate();
        for (temp_source, (src_temp_rate, src_radius, src_pos)) in temperature_sources {
            let src_falloff = entity::get_component(temp_source, temperature_src_falloff())
                .unwrap_or(DEFAULT_TEMPERATURE_SRC_FALLOFF);
            for (warm_body, (_current_temp, body_pos)) in &warm_bodies {
                if let Some(temp_effect_mult) = get_temperature_effect_multiplier(
                    *warm_body,
                    temp_source,
                    *body_pos,
                    src_pos,
                    src_radius,
                    src_falloff,
                ) {
                    entity::mutate_component(*warm_body, temperature(), |temp| {
                        *temp += temp_effect_mult * src_temp_rate * delta_time()
                    });
                }
            }
        }
    });
}

fn get_temperature_effect_multiplier(
    affected_body_id: EntityId,
    temp_source_id: EntityId,
    body_position: Vec3,
    source_position: Vec3,
    source_radius: f32,
    source_falloff: f32,
) -> Option<f32> {
    if affected_body_id == temp_source_id {
        return Some(1.); // if these are the same object, ignore exposure - return 1.
    }

    let dsq = body_position.distance_squared(source_position);
    if dsq > source_radius * source_radius {
        return None; // if the entities are too far apart, no effect
    } else if source_falloff <= 0.0001 {
        return Some(1.); // return full effect
    }

    // finally, do the falloff math if you absolutely must.
    Some(
        remap32(
            dsq.sqrt(),
            source_radius * (1. - source_falloff), // closest range - most effect
            source_radius,                         // furthest range - least effect
            1.,
            0.,
        )
        .clamp(0., 1.),
    )
}

fn remap32(value: f32, low1: f32, high1: f32, low2: f32, high2: f32) -> f32 {
    low2 + (value - low1) * (high2 - low2) / (high1 - low1)
}
