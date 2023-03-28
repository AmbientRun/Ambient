use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        primitives::cube,
        rendering::color,
        transform::{lookat_center, translation},
    },
    concepts::make_perspective_infinite_reverse_camera,
    prelude::*,
};
use components::{grid_side_length, grid_x, grid_y};

#[main]
pub async fn main() -> ResultEmpty {
    entity::wait_for_component(entity::synchronized_resources(), grid_side_length()).await;

    let side_length = entity::get_component(entity::synchronized_resources(), grid_side_length())
        .context("no side length on synchronized resources")?;

    let id = Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with_default(main_scene())
        .with(translation(), Vec3::ONE * 5.)
        .with(lookat_center(), vec3(0., 0., 0.))
        .spawn();

    on(event::FRAME, move |_| {
        entity::set_component(
            id,
            translation(),
            Quat::from_rotation_z(time() * 0.2) * Vec3::ONE * 10.,
        );
        OkEmpty
    });

    query((cube(), grid_x(), grid_y()))
        .build()
        .each_frame(move |entities| {
            for (id, (_, x, y)) in entities {
                let grid_cell = glam::ivec2(x - side_length, y - side_length);
                entity::mutate_component(id, translation(), |v| {
                    v.z = (x as f32 + y as f32 + time()).sin() - 0.5 * grid_cell.as_vec2().length();
                });

                let s = (time().sin() + 1.0) / 2.0;
                let t = (((x + y) as f32).sin() + 1.0) / 2.0;
                entity::set_component(id, color(), vec3(s, 1.0 - s, t).extend(1.0));
            }
        });

    OkEmpty
}
