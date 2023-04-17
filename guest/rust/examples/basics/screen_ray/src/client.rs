use ambient_api::{
    components::core::{
        app::{main_scene, window_logical_size},
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
        let input = player::get_raw_input();

        let window_size =
            entity::get_component(entity::resources(), window_logical_size()).unwrap();

        // Calculate normalized device coordinates
        let ndc_x = (2.0 * input.mouse_position.x / window_size.x as f32) - 1.0;
        let ndc_y = 1.0 - (2.0 * input.mouse_position.y / window_size.y as f32);

        let ndc = vec2(ndc_x, ndc_y);
        let ray = camera::screen_ray(camera, ndc);

        // Send screen ray to server
        messages::Input {
            ray_origin: ray.origin,
            ray_dir: ray.dir,
        }
        .send_server_unreliable();
    });
}
