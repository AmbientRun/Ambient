use ambient_api::prelude::*;

use packages::tangent_schema::concepts::VehicleClass;

use packages::this::components::is_tank;

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

        density: 15.0,
        cube_collider: Vec3::new(0.6, 1.0, 0.2),
        max_health: 150.0,

        offsets,
        k_p: 200.0,
        k_d: -400.0,
        target: 3.0,
        max_strength: 25.0,

        forward_force: 30.0,
        backward_force: -5.0,
        forward_offset: vec2(0.0, Y_DISTANCE),
        side_force: 50.0 / 100.0,
        side_offset: vec2(0.0, -Y_DISTANCE),

        jump_force: 20.0,
        pitch_strength: 10.0,
        turning_strength: 20.0,

        jump_timeout: Duration::from_secs_f32(2.0),

        linear_strength: 0.8,
        angular_strength: 0.4,
        angular_delay: Duration::from_secs_f32(0.25),

        model_url: packages::kenney_space_kit::assets::url("craft_miner.glb/models/main.json"),
        model_scale: 0.5,
    }
    .make()
    .with(is_tank(), ())
    .spawn();
}
