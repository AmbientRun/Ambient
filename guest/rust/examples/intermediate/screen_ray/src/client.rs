use ambient_api::{
    core::{messages::Frame, rendering::components::color, transform::components::translation},
    element::{use_module_message, use_state},
    prelude::*,
};
use packages::{
    orbit_camera::concepts::OrbitCamera,
    this::messages::{Input, WorldPosition},
};

#[main]
pub fn main() {
    let camera = OrbitCamera {
        is_orbit_camera: (),
        lookat_target: Vec3::ZERO,
        optional: default(),
    }
    .make()
    .spawn();

    Frame::subscribe(move |_| {
        let input = input::get();
        let ray = camera::screen_position_to_world_ray(camera, input.mouse_position);

        // Send screen ray to server
        Input {
            ray_origin: ray.origin,
            ray_dir: ray.dir,
        }
        .send_server_unreliable();
    });

    WorldPositionDisplay::el(camera).spawn_interactive();
}

#[element_component]
fn WorldPositionDisplay(hooks: &mut Hooks, camera: EntityId) -> Element {
    let (position, set_position) = use_state(hooks, None);
    use_module_message::<WorldPosition>(hooks, move |_, _, msg| {
        set_position(Some(msg.position));
    });

    position
        .map(|position| {
            Text::el(position.to_string())
                .with(
                    translation(),
                    camera::world_to_screen(camera, position).extend(0.0),
                )
                .with(color(), Vec4::ONE)
        })
        .unwrap_or_default()
}
