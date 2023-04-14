use ambient_api::{
    components::core::{
        app::{main_scene, window_physical_size},
    },
    camera::screen_ray,
    prelude::*
};

#[main]
pub fn main() {
    ambient_api::messages::Frame::subscribe(move |_| {
        let input = player::get_raw_input();

        let window_size = entity::get_component(entity::resources(), window_physical_size()).unwrap();

        // Calculate normalized device coordinates
        let ndc_x = (2.0 * input.mouse_position.x / window_size.x as f32) - 1.0;
        let ndc_y = 1.0 - (2.0 * input.mouse_position.y / window_size.y as f32);

        let ndc = vec2(ndc_x, ndc_y);
        let ray = screen_ray(ndc);

        // Send screen ray to server
        messages::Input{
            ray_origin: ray.origin,
            ray_dir: ray.dir,
        }.send_server_unreliable();
    });
}
