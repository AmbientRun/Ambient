use ambient_api::{
    core::{
        player::components::{is_player, user_id},
        rendering::components::{double_sided, local_bounding_aabb_max},
        transform::components::local_to_world,
    },
    element::{use_entity_component, use_query},
    prelude::*,
};

use packages::this::components::{height_offset, text_size};

#[main]
pub fn main() {
    Nameplates.el().spawn_interactive();
}

#[element_component]
fn Nameplates(hooks: &mut Hooks) -> Element {
    let players = use_query(hooks, is_player());

    Group::el(players.into_iter().map(|player| Nameplate::el(player.0)))
}

#[element_component]
fn Nameplate(hooks: &mut Hooks, player_id: EntityId) -> Element {
    let Some(camera_id) = camera::get_active(None) else {
        return Element::new();
    };

    let Some(camera_inv_view) = use_entity_component(hooks, camera_id, local_to_world()) else {
        return Element::new();
    };

    let (_, camera_rotation, _) = camera_inv_view.to_scale_rotation_translation();
    let camera_rotation_z = camera_rotation.to_euler(glam::EulerRot::ZYX).0;

    let user_id =
        use_entity_component(hooks, player_id, user_id()).unwrap_or_else(|| "unknown".to_string());

    let control_of_entity = use_entity_component(
        hooks,
        player_id,
        packages::game_object::player::components::control_of_entity(),
    );
    let entity_id = control_of_entity.unwrap_or(player_id);

    let Some(entity_ltw) = use_entity_component(hooks, entity_id, local_to_world()) else {
        return Element::new();
    };

    let (_, _, entity_translation) = entity_ltw.to_scale_rotation_translation();

    let height_offset_value = use_entity_component(hooks, entity_id, height_offset());
    let local_bounding_z =
        use_entity_component(hooks, entity_id, local_bounding_aabb_max()).map(|m| m.z);
    let height_offset = height_offset_value.or(local_bounding_z).unwrap_or(2.0);

    let text_size = use_entity_component(hooks, entity_id, text_size()).unwrap_or(2.0);

    let nameplate_translation = entity_translation + height_offset * Vec3::Z;
    let nameplate_rotation =
        Quat::from_rotation_z(camera_rotation_z) * Quat::from_rotation_x(90f32.to_degrees());

    Element::new()
        .children(vec![LayoutFreeCenter::el(
            Text3D::el(user_id, text_size).with(double_sided(), true),
            true,
            true,
        )])
        .with(
            local_to_world(),
            Mat4::from_scale_rotation_translation(
                Vec3::ONE,
                nameplate_rotation,
                nameplate_translation,
            ),
        )
}
