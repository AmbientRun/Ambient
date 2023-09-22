use ambient_api::{core::transform::components::local_to_world, prelude::*};

use crate::packages::tangent_schema::vehicle::components as vc;

pub fn calculate_aim_position(vehicle_id: EntityId, input_aim_direction: Vec2, range: f32) -> Vec3 {
    let aim_rotation = Quat::from_rotation_x(input_aim_direction.y.to_radians())
        * Quat::from_rotation_z(input_aim_direction.x.to_radians());

    let mut position = Vec3::ZERO;
    for weapon_id in
        entity::get_component(vehicle_id, vc::aimable_weapon_refs()).unwrap_or_default()
    {
        let weapon_ltw = entity::get_component(weapon_id, local_to_world()).unwrap_or_default();
        position += weapon_ltw.transform_point3(Vec3::ZERO);
    }

    let mut vehicle_ltw = entity::get_component(vehicle_id, local_to_world()).unwrap_or_default();
    vehicle_ltw.w_axis = position.extend(1.0);
    vehicle_ltw.transform_point3(aim_rotation * range * -Vec3::Y)
}
