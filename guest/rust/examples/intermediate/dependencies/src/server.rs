use ambient_api::{
    core::{
        app::components::main_scene,
        camera::{
            components::aspect_ratio_from_window,
            concepts::make_perspective_infinite_reverse_camera,
        },
        transform::components::{lookat_target, translation},
    },
    prelude::*,
};
use embers::{
    ambient_example_deps_assets::types::SpinDirection,
    deps_code::{
        components::{spawned_by_us, spin_direction},
        messages::Spawn,
    },
};

#[main]
pub async fn main() {
    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with_default(main_scene())
        .with(translation(), Vec3::ONE * 5.)
        .with(lookat_target(), vec3(0., 0., 0.))
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
