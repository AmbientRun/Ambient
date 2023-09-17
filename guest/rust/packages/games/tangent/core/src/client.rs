use ambient_api::{
    core::{
        app::components::name, physics::components::linear_velocity,
        transform::components::rotation,
    },
    prelude::*,
};
use packages::tangent_schema::{messages::Input, vehicle::components as vc};

#[main]
pub fn main() {
    // HACK: despawn all wheels on spawn
    spawn_query(name()).bind(|entities| {
        for (id, name) in entities {
            if name.starts_with("wheel") {
                entity::despawn(id);
            }
        }
    });

    query((rotation(), linear_velocity()))
        .requires(vc::player_ref())
        .each_frame(|vehicles| {
            for (id, (rot, lv)) in vehicles {
                entity::add_component(id, vc::speed_kph(), lv.dot(rot * -Vec3::Y) * 3.6);
            }
        });

    fixed_rate_tick(Duration::from_millis(20), |_| {
        if !input::is_game_focused() {
            return;
        }

        let input = input::get();
        let direction = {
            let mut direction = Vec2::ZERO;
            if input.keys.contains(&KeyCode::W) {
                direction.y += 1.;
            }
            if input.keys.contains(&KeyCode::S) {
                direction.y -= 1.;
            }
            if input.keys.contains(&KeyCode::A) {
                direction.x -= 1.;
            }
            if input.keys.contains(&KeyCode::D) {
                direction.x += 1.;
            }
            direction
        };
        Input {
            direction,
            jump: input.keys.contains(&KeyCode::Space),
        }
        .send_server_unreliable();
    });
}
