use ambient_api::global::time;
use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        primitives::cube,
        rendering::color,
        transform::{lookat_target, translation},
    },
    concepts::make_perspective_infinite_reverse_camera,
    prelude::*,
};
use components::{grid_side_length, grid_x, grid_y};

#[main]
pub async fn main() {
    let side_length =
        entity::wait_for_component(entity::synchronized_resources(), grid_side_length()).await;

    let id = Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with_default(main_scene())
        .with(translation(), Vec3::ONE * 5.)
        .with(lookat_target(), vec3(0., 0., 0.))
        .spawn();

    let start_time = time();

    ambient_api::messages::Frame::subscribe(move |_| {
        let t = time() - start_time;
        entity::set_component(
            id,
            translation(),
            Quat::from_rotation_z(t.as_secs_f32() * 0.2) * Vec3::ONE * 10.,
        );
    });

    query((cube(), grid_x(), grid_y())).each_frame(move |entities| {
        for (id, (_, x, y)) in entities {
            let grid_cell = glam::ivec2(x - side_length, y - side_length);
            let t = (time() - start_time).as_secs_f32();
            entity::mutate_component(id, translation(), |v| {
                v.z = (x as f32 + y as f32 + t).sin() - 0.5 * grid_cell.as_vec2().length();
            });

            let s = (t.sin() + 1.0) / 2.0;
            let t = (((x + y) as f32).sin() + 1.0) / 2.0;
            entity::set_component(id, color(), vec3(s, 1.0 - s, t).extend(1.0));
        }
    });
}
