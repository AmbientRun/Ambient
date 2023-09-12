use std::f32::consts::PI;

use ambient_api::{
    core::{
        player::components::is_player,
        primitives::components::cube,
        rendering::components::color,
        transform::components::{scale, translation},
    },
    prelude::*,
};

mod constants;
use constants::*;

use packages::{
    orbit_camera::concepts::{OrbitCamera, OrbitCameraOptional},
    this::{
        components::{cell, cells, owned_by},
        messages::Input,
    },
};

#[main]
pub fn main() {
    const CUBE_SIZE: f32 = 0.6;
    const SIZE_F32: f32 = SIZE as f32;

    OrbitCamera {
        is_orbit_camera: (),
        lookat_target: Vec3::ZERO,
        optional: OrbitCameraOptional {
            camera_distance: Some(SIZE_F32),
            camera_angle: Some(vec2(PI, 60f32.to_radians())),
            ..default()
        },
    }
    .make()
    .spawn();

    let cell_entities = {
        let mut cells = Vec::new();
        let centre = vec3((SIZE / 2) as f32, (SIZE / 2) as f32, 0.);

        for y in 0..SIZE {
            for x in 0..SIZE {
                let id = Entity::new()
                    .with(cube(), ())
                    .with(translation(), vec3(x as f32, y as f32, 0.) - centre)
                    .with(scale(), vec3(CUBE_SIZE, CUBE_SIZE, CUBE_SIZE))
                    .with(color(), DEFAULT_COLOR)
                    .spawn();
                cells.push(id);
            }
        }
        cells
    };
    entity::add_component(
        entity::synchronized_resources(),
        cells(),
        cell_entities.clone(),
    );

    spawn_query(is_player()).bind(|ids| {
        for (id, _) in ids {
            entity::add_component(id, cell(), 0);
        }
    });

    despawn_query(is_player()).bind(|ids| {
        let cells = entity::get_component(entity::synchronized_resources(), cells()).unwrap();

        for (id, _) in ids {
            for cell in &cells {
                if entity::get_component(*cell, owned_by()) == Some(id) {
                    entity::remove_component(*cell, owned_by());
                }
            }
        }
    });

    Input::subscribe(move |ctx, msg| {
        let Some(player_id) = ctx.client_entity_id() else {
            return;
        };
        let Some(cell) = entity::get_component(player_id, cell()) else {
            return;
        };

        let size = SIZE as i32;

        let mut x = cell % size;
        let mut y = cell / size;

        if msg.left {
            x = (x + size - 1) % size;
        }
        if msg.right {
            x = (x + 1) % size;
        }
        if msg.up {
            y = (y + size - 1) % size;
        }
        if msg.down {
            y = (y + 1) % size;
        }

        let cell = y * size + x;
        entity::set_component(player_id, self::cell(), cell);

        if msg.capture {
            entity::add_component_if_required(cell_entities[cell as usize], owned_by(), player_id);
        }
    });
}
