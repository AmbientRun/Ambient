use std::collections::HashMap;

use ambient_api::{
    core::{
        app::components::main_scene,
        physics::components::{cube_collider, dynamic, mass, physics_controlled, plane_collider},
        primitives::components::cube,
        rendering::components::{
            cast_shadows, color, fog_color, fog_density, fog_height_falloff, light_diffuse, sky,
            sun, water,
        },
        transform::components::{rotation, scale, translation},
    },
    prelude::*,
    rand,
};

#[main]
pub async fn main() {
    // Make sky
    Entity::new().with(sky(), ()).spawn();

    // Make sun
    Entity::new()
        .with(sun(), 0.0)
        .with(rotation(), Default::default())
        .with(main_scene(), ())
        .with(light_diffuse(), Vec3::ONE)
        .with(fog_color(), vec3(0.88, 0.37, 0.34))
        .with(fog_density(), 0.01)
        .with(fog_height_falloff(), 0.1)
        .with(rotation(), Quat::from_rotation_y(190.0f32.to_radians()))
        .spawn();

    // Make water
    Entity::new()
        .with(water(), ())
        .with(physics_controlled(), ())
        .with(plane_collider(), ())
        .with(dynamic(), false)
        .with(scale(), Vec3::ONE * 4000.)
        .spawn();

    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    make_cubes(&mut rng);
}

fn make_cubes(rng: &mut dyn rand::RngCore) {
    const TARGET_CUBE_COUNT: usize = 2000;
    const CUBE_BOUNDS: f32 = 250.;
    const CUBE_MIN_SIZE: Vec3 = vec3(0.5, 0.5, 0.5);
    const CUBE_MAX_SIZE: Vec3 = vec3(5., 6., 15.);
    const MASS_MULTIPLIER: f32 = 10.;

    let mut grid = Grid::default();
    while grid.size() < TARGET_CUBE_COUNT {
        let pos = rng.gen::<Vec2>() * (2. * CUBE_BOUNDS) - CUBE_BOUNDS;
        if exclude(pos) {
            continue;
        }

        let base_size = vec3(rng.gen(), rng.gen(), rng.gen());
        let size = base_size * (CUBE_MAX_SIZE - CUBE_MIN_SIZE) + CUBE_MIN_SIZE;
        let radius = size.xy().max_element();
        let volume = size.dot(Vec3::ONE);
        if grid.would_collide(pos, radius) {
            continue;
        }

        Entity::new()
            .with(cube(), ())
            .with(cast_shadows(), ())
            // Properties
            .with(translation(), vec3(pos.x, pos.y, size.z / 2.))
            .with(scale(), size)
            .with(color(), rng.gen::<Vec3>().extend(1.))
            // Physics
            .with(physics_controlled(), ())
            .with(cube_collider(), Vec3::ONE)
            .with(dynamic(), true)
            .with(mass(), volume * MASS_MULTIPLIER)
            .spawn();

        grid.add(pos, radius);
    }
}

fn exclude(pos: Vec2) -> bool {
    if pos.length() < 20. {
        return true;
    }

    false
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
