use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        player::local_user_id,
        transform::{lookat_center, rotation, translation},
    },
    concepts::make_perspective_infinite_reverse_camera,
    messages::Frame,
    prelude::*,
};
use components::player_vehicle;

#[main]
pub fn main() {
    let camera_id = Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with_default(main_scene())
        .with(translation(), vec3(5., 5., 2.))
        .with(lookat_center(), vec3(0., 0., 1.))
        .spawn();

    Frame::subscribe(move |_| {
        const CAMERA_OFFSET: Vec3 = vec3(0., 2.5, 1.);

        let player_id = local_entity_id();
        let Some(vehicle_id) = entity::get_component(player_id, player_vehicle()) else { return; };
        let Some(vehicle_position) = entity::get_component(vehicle_id, translation()) else { return; };
        let Some(vehicle_rotation) = entity::get_component(vehicle_id, rotation()) else { return; };

        let camera_position = vehicle_position + vehicle_rotation * CAMERA_OFFSET;
        entity::set_component(camera_id, translation(), camera_position);
        entity::set_component(
            camera_id,
            lookat_center(),
            camera_position + vehicle_rotation * -Vec3::Y,
        )
    });
}

// TODO: add to API
fn local_entity_id() -> EntityId {
    player::get_by_user_id(&entity::get_component(entity::resources(), local_user_id()).unwrap())
        .unwrap()
}
