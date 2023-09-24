use ambient_api::{
    core::{hierarchy::components::parent, transform::components::local_to_parent},
    prelude::*,
};

use packages::{
    gun_laser::concepts::{GunLaser, GunLaserOptional},
    tangent_schema::{concepts::VehicleDef, vehicle::components as vc},
};

#[main]
pub fn main() {
    make_allrounder();
    make_speedy();
    make_tank();
}

fn make_allrounder() {
    const X_DISTANCE: f32 = 0.1;
    const Y_DISTANCE: f32 = 0.4;

    let offsets = vec![
        vec2(-X_DISTANCE, -Y_DISTANCE),
        vec2(X_DISTANCE, -Y_DISTANCE),
        vec2(X_DISTANCE, Y_DISTANCE),
        vec2(-X_DISTANCE, Y_DISTANCE),
    ];

    let def = VehicleDef {
        density: 12.0,
        cube_collider: Vec3::new(0.6, 1.0, 0.2),
        max_health: 100.0,

        offsets,
        k_p: 300.0,
        k_d: -600.0,
        target: 1.75,
        max_strength: 25.0,

        forward_force: 40.0,
        backward_force: -10.0,
        forward_offset: vec2(0.0, Y_DISTANCE),
        side_force: 75.0 / 100.0,
        side_offset: vec2(0.0, -Y_DISTANCE),

        jump_force: 50.0,
        pitch_strength: 10.0,
        turning_strength: 20.0,

        aim_direction_limits: Vec2::ONE * 15f32,

        jump_timeout: Duration::from_secs_f32(2.0),

        linear_strength: 0.8,
        angular_strength: 0.4,
        angular_delay: Duration::from_secs_f32(0.25),

        is_def: (),
        name: "Thunderstrike".to_string(),
        model_url: packages::kenney_space_kit::assets::url("craft_speederA.glb/models/main.json"),
        model_scale: 0.5,
    }
    .spawn();

    spawn_query(vc::def_ref())
        .requires(vc::is_vehicle())
        .bind(move |vehicles| {
            for (vehicle_id, def_ref) in vehicles {
                if def_ref != def {
                    continue;
                }

                let weapon_ids = (-1..=1)
                    .step_by(2)
                    .map(|i| {
                        GunLaser {
                            is_gun_laser: (),
                            local_to_world: default(),
                            damage: 20.0,
                            time_between_shots: Duration::from_millis(250),
                            optional: GunLaserOptional {
                                translation: Some(vec3(i as f32 * 0.05, -0.45, 0.0)),
                                rotation: Some(default()),
                                ..default()
                            },
                        }
                        .make()
                        .with(parent(), vehicle_id)
                        .with(local_to_parent(), default())
                        .spawn()
                    })
                    .collect();

                entity::add_component(vehicle_id, vc::aimable_weapon_refs(), weapon_ids);
            }
        });
}

fn make_speedy() {
    const X_DISTANCE: f32 = 0.1;
    const Y_DISTANCE: f32 = 0.4;

    let offsets = vec![
        vec2(-X_DISTANCE, -Y_DISTANCE),
        vec2(X_DISTANCE, -Y_DISTANCE),
        vec2(X_DISTANCE, Y_DISTANCE),
        vec2(-X_DISTANCE, Y_DISTANCE),
    ];

    let def = VehicleDef {
        density: 10.0,
        cube_collider: Vec3::new(0.6, 1.0, 0.2),
        max_health: 70.0,

        offsets,
        k_p: 400.0,
        k_d: -800.0,
        target: 1.75,
        max_strength: 25.0,

        forward_force: 50.0,
        backward_force: -20.0,
        forward_offset: vec2(0.0, Y_DISTANCE),
        side_force: 100.0 / 100.0,
        side_offset: vec2(0.0, -Y_DISTANCE),

        jump_force: 70.0,
        pitch_strength: 10.0,
        turning_strength: 20.0,

        jump_timeout: Duration::from_secs_f32(2.0),

        aim_direction_limits: Vec2::ONE * 10f32,

        linear_strength: 0.8,
        angular_strength: 0.4,
        angular_delay: Duration::from_secs_f32(0.25),

        is_def: (),
        name: "Swiftshadow".to_string(),
        model_url: packages::kenney_space_kit::assets::url("craft_racer.glb/models/main.json"),
        model_scale: 0.5,
    }
    .spawn();

    spawn_query(vc::def_ref())
        .requires(vc::is_vehicle())
        .bind(move |vehicles| {
            for (vehicle_id, def_ref) in vehicles {
                if def_ref != def {
                    continue;
                }

                let weapon_id = GunLaser {
                    is_gun_laser: (),
                    local_to_world: default(),
                    damage: 20.0,
                    time_between_shots: Duration::from_millis(500),
                    optional: GunLaserOptional {
                        translation: Some(vec3(0.0, -0.55, 0.1)),
                        rotation: Some(default()),
                        ..default()
                    },
                }
                .make()
                .with(parent(), vehicle_id)
                .with(local_to_parent(), default())
                .spawn();

                entity::add_component(vehicle_id, vc::aimable_weapon_refs(), vec![weapon_id]);
            }
        });
}

fn make_tank() {
    const X_DISTANCE: f32 = 0.1;
    const Y_DISTANCE: f32 = 0.4;

    let offsets = vec![
        vec2(-X_DISTANCE, -Y_DISTANCE),
        vec2(X_DISTANCE, -Y_DISTANCE),
        vec2(X_DISTANCE, Y_DISTANCE),
        vec2(-X_DISTANCE, Y_DISTANCE),
    ];

    let def = VehicleDef {
        density: 15.0,
        cube_collider: Vec3::new(0.6, 1.0, 0.2),
        max_health: 150.0,

        offsets,
        k_p: 200.0,
        k_d: -400.0,
        target: 2.0,
        max_strength: 50.0,

        forward_force: 35.0,
        backward_force: -5.0,
        forward_offset: vec2(0.0, Y_DISTANCE),
        side_force: 60.0 / 100.0,
        side_offset: vec2(0.0, -Y_DISTANCE),

        jump_force: 30.0,
        pitch_strength: 10.0,
        turning_strength: 30.0,

        jump_timeout: Duration::from_secs_f32(2.0),

        aim_direction_limits: Vec2::ONE * 20f32,

        linear_strength: 0.8,
        angular_strength: 0.4,
        angular_delay: Duration::from_secs_f32(0.25),

        is_def: (),
        name: "Ironclad".to_string(),
        model_url: packages::kenney_space_kit::assets::url("craft_miner.glb/models/main.json"),
        model_scale: 0.5,
    }
    .spawn();

    spawn_query(vc::def_ref())
        .requires(vc::is_vehicle())
        .bind(move |vehicles| {
            for (vehicle_id, def_ref) in vehicles {
                if def_ref != def {
                    continue;
                }

                let weapon_id = GunLaser {
                    is_gun_laser: (),
                    local_to_world: default(),
                    damage: 60.0,
                    time_between_shots: Duration::from_millis(1250),
                    optional: GunLaserOptional {
                        translation: Some(vec3(0.0, -0.45, 0.05)),
                        rotation: Some(default()),
                        ..default()
                    },
                }
                .make()
                .with(parent(), vehicle_id)
                .with(local_to_parent(), default())
                .spawn();

                entity::add_component(vehicle_id, vc::aimable_weapon_refs(), vec![weapon_id]);
            }
        });
}
