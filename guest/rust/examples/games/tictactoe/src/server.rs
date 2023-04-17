use ambient_api::{
    components::core::{
        self,
        app::main_scene,
        camera::aspect_ratio_from_window,
        primitives::cube,
        rendering::color,
        transform::{lookat_center, scale, translation},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    prelude::*,
};

mod constants;
use constants::*;

#[main]
pub fn main() {
    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), entity::resources())
        .with_default(main_scene())
        .with(translation(), vec3(SIZE as f32, SIZE as f32, SIZE as f32))
        .with(
            lookat_center(),
            vec3(SIZE as f32 / 2., SIZE as f32 / 2., 0.),
        )
        .spawn();

    let cells = {
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
        components::cells(),
        cells.clone(),
    );

    spawn_query(core::player::player()).bind(|ids| {
        for (id, _) in ids {
            entity::add_component(id, components::cell(), 0);
        }
    });

    despawn_query(core::player::player()).bind(|ids| {
        let cells =
            entity::get_component(entity::synchronized_resources(), components::cells()).unwrap();

        for (id, _) in ids {
            for cell in &cells {
                if entity::get_component(*cell, components::owned_by()) == Some(id) {
                    entity::remove_component(*cell, components::owned_by());
                }
            }
        }
    });

    messages::Input::subscribe(move |source, msg| {
        let Some(player_id) = source.client_entity_id() else { return; };
        let Some(cell) = entity::get_component(player_id, components::cell()) else { return; };

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
        entity::set_component(player_id, components::cell(), cell);

        if msg.capture {
            entity::add_component_if_required(
                cells[cell as usize],
                components::owned_by(),
                player_id,
            );
        }
    });
}
