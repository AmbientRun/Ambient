use ambient_api::{
    core::{
        camera::components::projection,
        messages::Frame,
        primitives::components::cube,
        rendering::components::color,
        transform::components::{local_to_world, translation},
    },
    element::{use_entity_component, use_query},
    prelude::*,
};
use packages::{orbit_camera::concepts::OrbitCamera, this::messages::Input};

#[main]
pub fn main() {
    let camera = OrbitCamera {
        is_orbit_camera: (),
        optional: default(),
    }
    .spawn();

    Frame::subscribe(move |_| {
        let (delta, input) = input::get_delta();
        if !entity::has_component(camera, projection()) {
            // HACK: workaround for the orbit_camera package not adding components to the camera
            // entity until the next frame. In future, the API functions will be fallible, allowing them
            // to return an error if the entity doesn't have the required components.
            return;
        }
        let ray = camera::screen_position_to_world_ray(camera, input.mouse_position);

        // Send screen ray to server
        Input {
            ray_origin: ray.origin,
            ray_dir: ray.dir,
            spawn: delta.mouse_buttons_released.contains(&MouseButton::Left),
        }
        .send_server_unreliable();
    });

    WorldPositionDisplays::el(camera).spawn_interactive();
}

#[element_component]
fn WorldPositionDisplays(hooks: &mut Hooks, camera: EntityId) -> Element {
    // Ensure that this element re-renders when the camera moves
    let Some(_) = use_entity_component(hooks, camera, local_to_world()) else {
        return Element::new();
    };
    // Ensure that this element re-renders when the cubes move
    let cubes = use_query(hooks, (cube(), translation()));

    Group::el(
        cubes
            .into_iter()
            .map(|(_, (_, translation))| WorldPositionDisplay::el(camera, translation)),
    )
}

#[element_component]
fn WorldPositionDisplay(_hooks: &mut Hooks, camera: EntityId, position_3d: Vec3) -> Element {
    let position_2d = camera::world_to_screen(camera, position_3d);
    Text::el(format!("3D: {position_3d}\n2D: {position_2d}"))
        .with(translation(), position_2d.xy().extend(0.0))
        .with(color(), Vec4::ONE)
}
