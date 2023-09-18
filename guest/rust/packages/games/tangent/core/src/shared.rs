use ambient_api::{core::transform::components::local_to_world, prelude::*};

pub fn calculate_aim_position(vehicle_id: EntityId, input_aim_direction: Vec2) -> Vec3 {
    let aim_rotation = Quat::from_rotation_x(input_aim_direction.y.to_radians())
        * Quat::from_rotation_z(input_aim_direction.x.to_radians());

    let vehicle_ltw = entity::get_component(vehicle_id, local_to_world()).unwrap_or_default();
    vehicle_ltw.transform_point3(aim_rotation * 1000.0 * -Vec3::Y)
}
