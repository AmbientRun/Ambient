use ambient_api::{
    core::{
        app::components::main_scene,
        camera::{
            components::aspect_ratio_from_window,
            concepts::make_perspective_infinite_reverse_camera,
        },
        player::components::is_player,
        primitives::components::cube,
        rendering::components::color,
        transform::{
            components::{lookat_target, scale, translation},
            concepts::make_transformable,
        },
    },
    prelude::*,
};

mod constants;
use constants::*;

use embers::ambient_example_tictactoe::{
    components::{cell, cells, owned_by},
    messages::Input,
};

#[main]
pub fn main() {
    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), entity::resources())
        .with_default(main_scene())
        .with(translation(), vec3(SIZE as f32, SIZE as f32, SIZE as f32))
        .with(
            lookat_target(),
            vec3(SIZE as f32 / 2., SIZE as f32 / 2., 0.),
        )
        .spawn();

    let cell_entities = {
        let mut cells = Vec::new();
        for y in 0..SIZE {
            for x in 0..SIZE {
                let id = Entity::new()
                    .with_merge(make_transformable())
                    .with_default(cube())
                    .with(translation(), vec3(x as f32, y as f32, 0.))
                    .with(scale(), vec3(0.6, 0.6, 0.6))
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

    Input::subscribe(move |source, msg| {
        let Some(player_id) = source.client_entity_id() else { return; };
        let Some(cell) = entity::get_component(player_id, cell()) else { return; };

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
