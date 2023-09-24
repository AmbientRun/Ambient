use ambient_api::{
    core::{hierarchy::components::parent, transform::components::local_to_parent},
    prelude::*,
};

use packages::{
    gun_laser::concepts::{GunLaser, GunLaserOptional},
    tangent_schema::{
        concepts::{VehicleClass, VehicleDef},
        vehicle::components as vc,
    },
    this::components::is_tank,
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

        name: "Tank".to_string(),
        description: "A juggernaut on the battlefield, built to withstand punishment and deal massive damage."
            .to_string(),
        icon_url: packages::this::assets::url("icon.png"),

        def_ref: VehicleDef {
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

            model_url: packages::kenney_space_kit::assets::url("craft_miner.glb/models/main.json"),
            model_scale: 0.5,
        }
        .make()
        .with(is_tank(), ())
        .spawn(),
    }
    .spawn();

    spawn_query(vc::def_ref())
        .requires(vc::is_vehicle())
        .bind(|vehicles| {
            for (vehicle_id, def_ref) in vehicles {
                if !entity::has_component(def_ref, is_tank()) {
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
