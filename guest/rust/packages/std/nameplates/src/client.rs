use ambient_api::{
    core::{
        hierarchy::components::parent,
        player::components::{is_player, user_id},
        rendering::components::double_sided,
        transform::components::{
            cylindrical_billboard_z, local_to_parent, local_to_world, rotation, translation,
        },
    },
    element::{use_entity_component, use_query, use_state},
    prelude::*,
};

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

    let Some(camera_inv_view) = use_entity_component(hooks, camera_id, local_to_world()).0 else {
        return Element::new();
    };

    let (_, camera_rotation, _) = camera_inv_view.to_scale_rotation_translation();
    let camera_rotation_z = camera_rotation.to_euler(glam::EulerRot::ZYX).0;

    let user_id = use_entity_component(hooks, player_id, user_id())
        .0
        .unwrap_or_else(|| "unknown".to_string());

    let control_of_entity = use_entity_component(
        hooks,
        player_id,
        packages::game_object::player::components::control_of_entity(),
    )
    .0;
    let entity_id = control_of_entity.unwrap_or(player_id);

    let Some(entity_ltw) = use_entity_component(hooks, entity_id, local_to_world()).0 else {
        return Element::new();
    };

    let (_, _, entity_translation) = entity_ltw.to_scale_rotation_translation();

    let nameplate_translation = entity_translation + vec3(0.0, 0.0, 0.5);
    let nameplate_rotation =
        Quat::from_rotation_z(camera_rotation_z) * Quat::from_rotation_x(90f32.to_degrees());

    Element::new()
        .children(vec![LayoutFreeCenter::el(
            Text3D::el(user_id, 2.0)
                .with(double_sided(), true)
                .init(width(), 1.0),
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
