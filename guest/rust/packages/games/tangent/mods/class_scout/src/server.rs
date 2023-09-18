use ambient_api::{
    core::{hierarchy::components::parent, transform::components::local_to_parent},
    prelude::*,
};

use packages::{
    gun_laser::concepts::{GunLaser, GunLaserOptional},
    tangent_schema::{
        concepts::VehicleClass, player::components as pc, vehicle::components as vc,
        weapon::messages::Fire,
    },
    this::components::is_scout,
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

    #[allow(clippy::eq_op)]
    VehicleClass {
        is_class: (),

        name: "Scout".to_string(),
        description: "Swift and elusive, ideal for hit-and-run tactics and recon missions."
            .to_string(),
        icon_url: packages::this::assets::url("icon.png"),

        density: 10.0,
        cube_collider: Vec3::new(0.6, 1.0, 0.2),
        max_health: 70.0,

        offsets,
        k_p: 400.0,
        k_d: -800.0,
        target: 2.0,
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

        model_url: packages::kenney_space_kit::assets::url("craft_racer.glb/models/main.json"),
        model_scale: 0.5,
    }
    .make()
    .with(is_scout(), ())
    .spawn();

    spawn_query(vc::player_ref())
        .requires(is_scout())
        .bind(|vehicles| {
            for (vehicle_id, _) in vehicles {
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

    query((vc::player_ref(), vc::aimable_weapon_refs()))
        .requires(is_scout())
        .each_frame(|vehicles| {
            for (_vehicle_id, (player_id, weapon_ids)) in vehicles {
                if let Some(true) = entity::get_component(player_id, pc::input_fire()) {
                    for weapon_id in weapon_ids {
                        Fire { weapon_id }.send_local_broadcast(false);
                    }
                }
            }
        });
}
