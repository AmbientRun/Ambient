use std::{
    collections::HashMap,
    f32::consts::{PI, TAU},
};

use ambient_api::{
    core::{
        app::components::main_scene,
        hierarchy::components::parent,
        model::components::model_from_url,
        physics::components::{cube_collider, dynamic, mass, physics_controlled, plane_collider},
        primitives::{
            components::{cube, quad},
            concepts::Capsule,
        },
        rendering::components::{cast_shadows, color, fog_density, light_diffuse, sky, sun},
        transform::components::{
            local_to_parent, local_to_world, mesh_to_local, mesh_to_world, rotation, scale,
            translation,
        },
    },
    ecs::GeneralQuery,
    prelude::*,
    rand,
};

use packages::{
    pickup_health::{components::is_health_pickup, concepts::HealthPickup},
    tangent_schema::{
        concepts::{Spawnpoint, Vehicle, VehicleDef},
        vehicle::components::is_vehicle,
        vehicle::def::components::is_def,
    },
};

mod shared;
use shared::LEVEL_RADIUS;

#[main]
pub async fn main() {
    // Make sky
    Entity::new().with(sky(), ()).spawn();

    // Make sun
    Entity::new()
        .with(sun(), 0.0)
        .with(rotation(), Quat::from_rotation_y(-45_f32.to_radians()))
        .with(main_scene(), ())
        .with(light_diffuse(), Vec3::ONE)
        .with(fog_density(), 0.)
        .with(main_scene(), ())
        .spawn();

    // Make ground
    Entity::new()
        .with(quad(), ())
        .with(physics_controlled(), ())
        .with(plane_collider(), ())
        .with(dynamic(), false)
        .with(scale(), Vec3::ONE * 4000.)
        .with(color(), vec4(0.93, 0.75, 0.83, 1.0))
        .spawn();

    // Spawn spawnpoints
    for (pos, radius, color) in shared::spawnpoints().iter().copied() {
        Spawnpoint {
            is_spawnpoint: (),
            radius,
            translation: pos,
            color: color.extend(1.0),
        }
        .make()
        .with_merge(Capsule {
            capsule_half_height: 0.1,
            ..Capsule::suggested()
        })
        .with(scale(), vec3(radius, radius, 1.0))
        .spawn();
    }

    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let mut grid = Grid::default();
    make_cubes(&mut rng, &mut grid);
    handle_pickups();
    handle_vehicles();
}

fn make_cubes(rng: &mut dyn rand::RngCore, grid: &mut Grid) {
    const TARGET_CUBE_COUNT: usize = 500;
    const CUBE_MIN_SIZE: Vec3 = vec3(0.5, 0.5, 2.0);
    const CUBE_MAX_SIZE: Vec3 = vec3(7., 8., 30.);
    const FADE_DISTANCE: f32 = 2.;

    let bright_colors = [vec4(0.5921569, 0.78039217, 0.8509804, 1.0)];

    let accent_colors = [
        vec4(0.15686275, 0.44313726, 0.9764706, 1.0),
        vec4(0.8392157, 0.35686275, 0.26666668, 1.0),
        vec4(0.77254903, 0.8156863, 0.25882354, 1.0),
        vec4(0.7490196, 0.12941177, 0.14901961, 1.0),
    ];

    // Spawn cubes until we hit the limit
    while grid.size() < TARGET_CUBE_COUNT {
        let position =
            shared::circle_point(rng.gen::<f32>() * TAU, rng.gen::<f32>() * LEVEL_RADIUS);

        let base_size = vec3(rng.gen(), rng.gen(), rng.gen());
        let size = base_size * (CUBE_MAX_SIZE - CUBE_MIN_SIZE) + CUBE_MIN_SIZE;
        let radius = size.xy().max_element();

        let level = shared::level(position);
        if level < radius {
            continue;
        }

        let probability = ((level - radius) / FADE_DISTANCE).clamp(0.0, 1.0);
        let sample = rng.gen::<f32>();
        if sample > probability {
            continue;
        }

        if grid.would_collide(position, radius) {
            continue;
        }

        let position_offset = vec3(0.0, 0.0, -CUBE_MIN_SIZE.z * 0.6);

        let accent_sample = rng.gen::<f32>();
        let color = if accent_sample > 0.3 {
            let accent_idx = match (position.x > 0.0, position.y > 0.0) {
                (true, true) => 3,
                (true, false) => 2,
                (false, true) => 1,
                (false, false) => 0,
            };
            accent_colors[accent_idx]
        } else {
            *bright_colors.choose(rng).unwrap()
        };

        make_cube(
            position.extend(0.0) + position_offset,
            size,
            false,
            // TODO: In the words of another Ambient example:
            // This is a bit... odd
            color * 2.2,
            rng,
        );
        grid.add(position, radius);
    }

    // Make surrounding walls
    for i in 0..360 {
        let angle = (i as f32).to_radians();
        let radius = LEVEL_RADIUS + rng.gen::<f32>() * 10.0;
        let position = shared::circle_point(angle, radius);

        let size = vec3(1.5, 1.5, 10.) + rng.gen::<Vec3>() * vec3(1., 1., 20.);
        make_cube(position.extend(0.0), size, false, Vec4::ONE, rng);
    }
}

fn handle_pickups() {
    handle_respawnables(
        shared::spawnpoints().len() * 4,
        query(translation()).requires(is_health_pickup()).build(),
        Duration::from_secs(5),
        25.0,
        |translation| {
            HealthPickup {
                is_health_pickup: (),
                translation: translation.extend(1.0),
                rotation: Quat::IDENTITY,
            }
            .spawn();
        },
    )
}

fn handle_vehicles() {
    handle_respawnables(
        shared::spawnpoints().len() * 3,
        query(translation()).requires(is_vehicle()).build(),
        Duration::from_secs(30),
        40.0,
        move |translation| {
            let Some(def_id) = entity::get_all(is_def()).choose(&mut thread_rng()).copied() else {
                return;
            };

            let Some(def) = VehicleDef::get_spawned(def_id) else {
                return;
            };

            let vehicle_id = Vehicle {
                linear_velocity: default(),
                angular_velocity: default(),
                physics_controlled: (),
                dynamic: true,
                density: def.density,
                cube_collider: def.cube_collider,

                local_to_world: default(),
                translation: translation.extend(def.target),
                rotation: Quat::from_rotation_z(random::<f32>() * PI),

                is_vehicle: (),

                health: def.max_health,
                max_health: def.max_health,

                last_distances: def.offsets.iter().map(|_| 0.0).collect(),
                last_jump_time: game_time(),
                last_slowdown_time: game_time(),
                def_ref: def_id,

                input_direction: default(),
                input_jump: default(),
                input_fire: default(),
                input_aim_direction: default(),

                optional: default(),
            }
            .spawn();

            let _vehicle_model_id = Entity::new()
                .with(cast_shadows(), ())
                .with(model_from_url(), def.model_url)
                .with(local_to_world(), default())
                .with(local_to_parent(), default())
                .with(mesh_to_local(), default())
                .with(mesh_to_world(), default())
                .with(main_scene(), ())
                .with(scale(), Vec3::ONE * def.model_scale)
                .with(parent(), vehicle_id)
                .spawn();
        },
    )
}

fn handle_respawnables(
    count: usize,
    locate_query: GeneralQuery<Component<Vec3>>,
    respawn_time: Duration,
    distance_from_each_other: f32,
    spawn: impl Fn(Vec2) + 'static,
) {
    let distance_from_each_other_sqr = distance_from_each_other.powi(2);
    let respawn = move || {
        let rng = &mut thread_rng();

        let mut existing: Vec<_> = locate_query
            .evaluate()
            .into_iter()
            .map(|p| p.1.xy())
            .collect();
        loop {
            if existing.len() >= count {
                break;
            }

            let position =
                shared::circle_point(rng.gen::<f32>() * TAU, rng.gen::<f32>() * LEVEL_RADIUS);

            let level = shared::level(position.xy());
            if level > 0.0 {
                continue;
            }

            if existing.iter().any(|other_position| {
                other_position.xy().distance_squared(position) < distance_from_each_other_sqr
            }) {
                continue;
            }

            spawn(position);
            existing.push(position);
        }
    };

    respawn();
    fixed_rate_tick(respawn_time, move |_| respawn());
}

fn make_cube(pos: Vec3, size: Vec3, dynamic: bool, color: Vec4, rng: &mut dyn RngCore) -> EntityId {
    const MASS_MULTIPLIER: f32 = 10.;

    let volume = size.dot(Vec3::ONE);
    let pitch_amplitude: f32 = if dynamic { 5. } else { 60. };
    Entity::new()
        .with(cube(), ())
        .with(cast_shadows(), ())
        // Properties
        .with(translation(), pos + vec3(0.0, 0.0, size.z / 2.))
        .with(
            rotation(),
            Quat::from_rotation_x((rng.gen::<f32>() - 0.5) * pitch_amplitude.to_radians())
                * Quat::from_rotation_z(rng.gen::<f32>() * TAU),
        )
        .with(scale(), size)
        .with(self::color(), color)
        // Physics
        .with(physics_controlled(), ())
        .with(cube_collider(), Vec3::ONE)
        .with(self::dynamic(), dynamic)
        .with(mass(), volume * MASS_MULTIPLIER)
        .spawn()
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
                if (box_.pos - pos).length_squared() < (box_.radius + radius).powi(2) {
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
