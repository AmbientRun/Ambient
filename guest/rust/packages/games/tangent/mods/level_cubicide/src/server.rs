use std::collections::HashMap;

use ambient_api::{
    core::{
        app::components::main_scene,
        physics::components::{cube_collider, dynamic, mass, physics_controlled, plane_collider},
        primitives::components::{cube, quad},
        rendering::components::{
            cast_shadows, color, fog_color, fog_density, fog_height_falloff, light_diffuse, sky,
            sun,
        },
        transform::components::{rotation, scale, translation},
    },
    prelude::*,
    rand,
};

mod shared;

#[main]
pub async fn main() {
    // Make sky
    Entity::new().with(sky(), ()).spawn();

    // Make sun
    let sky_color = vec3(0.11, 0.20, 0.27);
    Entity::new()
        .with(sun(), 0.0)
        .with(rotation(), Quat::from_rotation_y(10f32.to_radians()))
        .with(main_scene(), ())
        .with(light_diffuse(), sky_color * 2.)
        .with(fog_color(), sky_color)
        .with(fog_density(), 0.05)
        .with(fog_height_falloff(), 0.05)
        .spawn();

    // Make ground
    Entity::new()
        .with(quad(), ())
        .with(physics_controlled(), ())
        .with(plane_collider(), ())
        .with(dynamic(), false)
        .with(scale(), Vec3::ONE * 4000.)
        .with(color(), sky_color.extend(1.0))
        .spawn();

    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    make_cubes(&mut rng);
}

fn make_cubes(rng: &mut dyn rand::RngCore) {
    const TARGET_CUBE_COUNT: usize = 1000;
    const CUBE_BOUNDS: f32 = 125.;
    const CUBE_MIN_SIZE: Vec3 = vec3(0.5, 0.5, 0.5);
    const CUBE_MAX_SIZE: Vec3 = vec3(5., 6., 15.);
    const MASS_MULTIPLIER: f32 = 10.;

    let mut grid = Grid::default();
    while grid.size() < TARGET_CUBE_COUNT {
        let pos = rng.gen::<Vec2>() * (2. * CUBE_BOUNDS) - CUBE_BOUNDS;

        let base_size = vec3(rng.gen(), rng.gen(), rng.gen());
        let size = base_size * (CUBE_MAX_SIZE - CUBE_MIN_SIZE) + CUBE_MIN_SIZE;
        let radius = size.xy().max_element();

        if shared::level(pos) < radius {
            continue;
        }

        if grid.would_collide(pos, radius) {
            continue;
        }

        let volume = size.dot(Vec3::ONE);
        Entity::new()
            .with(cube(), ())
            .with(cast_shadows(), ())
            // Properties
            .with(translation(), vec3(pos.x, pos.y, size.z / 2.))
            .with(scale(), size)
            .with(color(), (rng.gen::<Vec3>() * 0.2).extend(1.))
            // Physics
            .with(physics_controlled(), ())
            .with(cube_collider(), Vec3::ONE)
            .with(dynamic(), true)
            .with(mass(), volume * MASS_MULTIPLIER)
            .spawn();

        grid.add(pos, radius);
    }
}

#[derive(Debug)]
struct Box {
    pos: Vec2,
    radius: f32,
}

#[derive(Default, Debug)]
struct Grid {
    cells: HashMap<IVec2, Vec<Box>>,
    size: usize,
}
impl Grid {
    const CELL_SIZE: f32 = 4.;

    fn position_to_cell(pos: Vec2) -> IVec2 {
        ivec2(
            (pos.x / Self::CELL_SIZE) as i32,
            (pos.y / Self::CELL_SIZE) as i32,
        )
    }

    fn add(&mut self, pos: Vec2, radius: f32) {
        self.cells
            .entry(Self::position_to_cell(pos))
            .or_default()
            .push(Box { pos, radius });

        self.size += 1;
    }

    fn would_collide(&self, pos: Vec2, radius: f32) -> bool {
        let cell = Self::position_to_cell(pos);
        const PROBE_OFFSETS: [IVec2; 9] = [
            ivec2(0, 0),
            ivec2(-1, 0),
            ivec2(1, 0),
            ivec2(0, -1),
            ivec2(0, 1),
            ivec2(-1, -1),
            ivec2(-1, 1),
            ivec2(1, -1),
            ivec2(1, 1),
        ];

        for offset in PROBE_OFFSETS {
            let cell = cell + offset;
            let Some(boxes) = self.cells.get(&cell) else {
                continue;
            };
            for box_ in boxes {
                if (box_.pos - pos).length() < box_.radius + radius {
                    return true;
                }
            }
        }

        false
    }

    fn size(&self) -> usize {
        self.size
    }
}
