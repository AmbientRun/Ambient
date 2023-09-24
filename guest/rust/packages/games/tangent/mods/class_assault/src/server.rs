use ambient_api::{
    core::{hierarchy::components::parent, transform::components::local_to_parent},
    prelude::*,
};

use packages::{
    gun_laser::concepts::{GunLaser, GunLaserOptional},
    tangent_schema::{
        concepts::{VehicleClass, VehicleData},
        vehicle::components as vc,
    },
    this::components::is_assault,
};

#[main]
pub fn main() {
    const X_DISTANCE: f32 = 0.1;
    const Y_DISTANCE: f32 = 0.4;

    let offsets = vec![
        vec2(-X_DISTANCE, -Y_DISTANCE),
        vec2(X_DISTANCE, -Y_DISTANCE),
        vec2(X_DISTANCE, Y_DISTANCE),
        vec2(-X_DISTANCE, Y_DISTANCE),
    ];

    VehicleClass {
        is_class: (),

        name: "Assault".to_string(),
        description: "A versatile choice for those who seek balance in speed, firepower, and maneuverability.".to_string(),
        icon_url: packages::this::assets::url("icon.png"),

        model_url: packages::kenney_space_kit::assets::url("craft_speederA.glb/models/main.json"),
        model_scale: 0.5,

        data_ref: VehicleData {
            density: 12.0,
            cube_collider: Vec3::new(0.6, 1.0, 0.2),
            max_health: 100.0,

            offsets,
            k_p: 300.0,
            k_d: -600.0,
            target: 2.5,
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
        }
        .make()
        .with(is_assault(), ())
        .spawn()
    }
    .spawn();

    spawn_query(vc::data_ref())
        .requires(vc::is_vehicle())
        .bind(|vehicles| {
            for (vehicle_id, data_ref) in vehicles {
                if !entity::has_component(data_ref, is_assault()) {
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
