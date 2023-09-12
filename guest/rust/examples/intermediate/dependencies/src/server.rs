use ambient_api::prelude::*;
use packages::{
    deps_assets::types::SpinDirection,
    deps_code::{
        components::{spawned_by_us, spin_direction},
        messages::Spawn,
    },
    orbit_camera::concepts::OrbitCamera,
};

#[main]
pub async fn main() {
    OrbitCamera {
        is_orbit_camera: (),
        lookat_target: Vec3::ZERO,
        optional: default(),
    }
    .make()
    .spawn();

    let mut last_update = game_time();
    query(spawned_by_us()).each_frame(move |r| {
        if game_time() - last_update < Duration::from_secs(1) {
            return;
        }
        for (id, _) in r {
            entity::mutate_component(id, spin_direction(), |d| {
                *d = match d {
                    SpinDirection::Forward => SpinDirection::Backward,
                    SpinDirection::Backward => SpinDirection::Forward,
                }
            });
        }
        last_update = game_time();
    });

    for i in 0..5 {
        Spawn::new((i + 1) as f32, SpinDirection::Forward).send_local_broadcast(false);
        sleep(0.25).await;
    }
}
