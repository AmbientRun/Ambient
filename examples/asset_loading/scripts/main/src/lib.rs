use elements_scripting_interface::{
    entity::{exists, set_position, set_rotation, spawn_template, wait_for_spawn}, *
};

pub mod components;
pub mod params;

#[main]
pub async fn main() -> EventResult {
    let cube_ref = ObjectRef::new("assets/Cube.glb/objects/main.json");
    let cube_uid = spawn_template(&cube_ref, Vec3::new(0.0, 0.0, 1.0), None, None, false);
    let cube_entity = wait_for_spawn(&cube_uid).await;

    on(event::FRAME, move |_| {
        if exists(cube_entity) {
            set_rotation(cube_entity, Quat::from_axis_angle(Vec3::X, time().sin()));
        }
        EventOk
    });

    EventOk
}
