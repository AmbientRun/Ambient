use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        transform::{lookat_center, translation},
    },
    concepts::make_perspective_infinite_reverse_camera,
    prelude::*,
};

#[main]
pub fn main() {
    let camera = Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with_default(main_scene())
        .with(translation(), Vec3::ONE * 5.)
        .with(lookat_center(), vec3(0., 0., 0.))
        .spawn();

    ambient_api::messages::Frame::subscribe(move |_| {
        let input = input::get();

        let ray = camera::screen_to_world_direction(camera, input.mouse_position);

        // Send screen ray to server
        messages::Input {
            ray_origin: ray.origin,
            ray_dir: ray.dir,
        }
        .send_server_unreliable();
    });
}
