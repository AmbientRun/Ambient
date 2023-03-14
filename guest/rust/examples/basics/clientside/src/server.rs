use ambient_api::{
    components::core::{primitives::cube, rendering::color, transform::translation},
    concepts::make_transformable,
    prelude::*,
};

use crate::components::{grid_x, grid_y};

#[main]
pub async fn main() -> EventResult {
    const GRID_SIDE_LENGTH: i32 = 10;

    for y in 0..2 * GRID_SIDE_LENGTH + 1 {
        for x in 0..2 * GRID_SIDE_LENGTH + 1 {
            Entity::new()
                .with_merge(make_transformable())
                .with_default(cube())
                .with(grid_x(), x)
                .with(grid_y(), y)
                .with(color(), Vec4::ONE)
                .with(
                    translation(),
                    vec3(
                        (x - GRID_SIDE_LENGTH) as f32,
                        (y - GRID_SIDE_LENGTH) as f32,
                        0.0,
                    ),
                )
                .spawn();
        }
    }

    EventOk
}
