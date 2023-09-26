use std::f32::consts::PI;

use ambient_api::{
    core::{
        app::components::main_scene,
        hierarchy::components::parent,
        model::components::model_from_url,
        rendering::components::cast_shadows,
        transform::components::{
            local_to_parent, local_to_world, mesh_to_local, mesh_to_world, scale,
        },
    },
    prelude::*,
};

use packages::{
    tangent_schema::concepts::{Vehicle, VehicleDef},
    this::messages::VehicleSpawn,
};

#[main]
pub fn main() {
    VehicleSpawn::subscribe(|ctx, msg| {
        let Some(_sender_id) = ctx.local() else {
            return;
        };

        let Some(def) = VehicleDef::get_spawned(msg.def_id) else {
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
            translation: msg.position + Vec3::Z * (def.target * 2.0),
            rotation: Quat::from_rotation_z(random::<f32>() * PI),

            is_vehicle: (),

            health: def.max_health,
            max_health: def.max_health,

            last_distances: def.offsets.iter().map(|_| 0.0).collect(),
            last_jump_time: game_time(),
            last_slowdown_time: game_time(),
            def_ref: msg.def_id,

            input_direction: default(),
            input_jump: default(),
            input_fire: default(),
            input_aim_direction: default(),

            optional: default(),
        }
        .make()
        .with(packages::nameplates::components::height_offset(), 0.5)
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
    });
}
