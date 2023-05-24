use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        transform::{lookat_target, translation},
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
        .with(lookat_target(), vec3(0., 0., 0.))
        .spawn();

    ambient_api::messages::Frame::subscribe(move |_| {
        let input = input::get();
        let ray = camera::screen_position_to_world_ray(camera, input.mouse_position);

        // Send screen ray to server
        messages::Input {
            ray_origin: ray.origin,
            ray_dir: ray.dir,
        }
        .send_server_unreliable();
    });

    WorldPosition::el(camera).spawn_interactive();
}

#[element_component]
fn WorldPosition(hooks: &mut Hooks, camera: EntityId) -> Element {
    let (position, set_position) = hooks.use_state(None);
    hooks.use_module_message::<messages::WorldPosition>(move |_, _, msg| {
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
