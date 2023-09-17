use ambient_api::{
    core::{
        app::components::name, physics::components::linear_velocity,
        transform::components::rotation,
    },
    once_cell::sync::Lazy,
    prelude::*,
};
use packages::tangent_schema::{
    messages::{Input, OnCollision},
    vehicle::client::components as vcc,
    vehicle::components as vc,
};

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
                entity::add_component(id, vcc::speed_kph(), lv.dot(rot * -Vec3::Y) * 3.6);
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

    handle_collisions();
}

fn handle_collisions() {
    static SOUNDS: Lazy<[Vec<String>; 3]> = Lazy::new(|| {
        let url = |ty, idx| {
            packages::kenney_impact_sounds::assets::url(&format!("impactPlate_{ty}_{idx:0>3}.ogg"))
        };

        ["light", "medium", "heavy"].map(|ty| (0..5).map(|idx| url(ty, idx)).collect())
    });

    OnCollision::subscribe(|ctx, msg| {
        if !ctx.server() {
            return;
        }

        let impact_type = match msg.speed {
            speed if speed < 5. => 0,
            speed if speed < 10. => 1,
            _ => 2,
        };

        let sound = SOUNDS[impact_type].choose(&mut thread_rng()).unwrap();
        audio::SpatialAudioPlayer::oneshot(msg.position, sound);
    });
}
