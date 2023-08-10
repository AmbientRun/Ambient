use std::f32::consts::PI;

use ambient_api::{
    core::{
        app::components::{main_scene, name},
        camera::{
            components::{active_camera, aspect_ratio_from_window},
            concepts::make_perspective_infinite_reverse_camera,
        },
        player::components::user_id,
        rendering::components::outline_recursive,
        transform::components::{rotation, translation},
    },
    prelude::*,
};

use afps_schema::components::player_name;
use editor::{
    components::{
        camera_angle, editor_camera, in_editor, mouseover_entity, mouseover_position,
        selected_entity,
    },
    messages::{Input, ToggleEditor},
};

#[main]
pub fn main() {
    ToggleEditor::subscribe(|source, _| {
        let Some(id) = source.client_entity_id() else { return; };

        let in_editor = entity::mutate_component_with_default(id, in_editor(), true, |in_editor| {
            *in_editor = !*in_editor;
        });

        if in_editor {
            entity::add_component_if_required(id, player_name(), "Editor".to_string());
            let player_user_id = entity::get_component(id, user_id()).unwrap();
            let player_position = entity::get_component(id, translation()).unwrap_or_default();

            let camera_id = Entity::new()
                .with_merge(make_perspective_infinite_reverse_camera())
                .with(aspect_ratio_from_window(), EntityId::resources())
                .with_default(main_scene())
                .with(user_id(), player_user_id)
                .with(translation(), player_position + vec3(0.0, 0.0, 5.0))
                .with(camera_angle(), vec2(0.0, PI / 2.))
                .with(name(), "Editor Camera".to_string())
                .with(active_camera(), 10.0)
                .spawn();

            entity::add_component(id, editor_camera(), camera_id);
        } else {
            if let Some(camera_id) = entity::get_component(id, editor_camera()) {
                entity::remove_component(id, editor_camera());
                entity::despawn(camera_id);
            }
        }
    });

    Input::subscribe(|source, msg| {
        let Some(id) = source.client_entity_id() else { return; };
        if !entity::get_component(id, in_editor()).unwrap_or_default() {
            return;
        }

        let Some(camera_id) = entity::get_component(id, editor_camera()) else { return; };

        let angle = entity::mutate_component_with_default(
            camera_id,
            camera_angle(),
            vec2(0.0, -PI),
            |angle| {
                *angle += msg.aim_delta;
                angle.x = angle.x % PI;
                angle.y = angle.y.clamp(0., PI);
            },
        );

        let new_rotation = Quat::from_rotation_z(angle.x) * Quat::from_rotation_x(angle.y);
        entity::set_component(camera_id, rotation(), new_rotation);

        let movement = msg.movement.normalize_or_zero();
        let movement_speed = if msg.boost { 2.0 } else { 1.0 };

        entity::mutate_component(camera_id, translation(), |translation| {
            *translation += new_rotation * vec3(movement.x, 0.0, -movement.y) * movement_speed;
        });

        if let Some(hit) = physics::raycast_first(msg.ray_origin, msg.ray_direction) {
            entity::add_component(id, mouseover_position(), hit.position);
            entity::add_component(id, mouseover_entity(), hit.entity);
        } else {
            entity::remove_component(id, mouseover_position());
            entity::remove_component(id, mouseover_entity());
        }

        if let (Some(entity), true) = (entity::get_component(id, mouseover_entity()), msg.select) {
            if let Some(selected) = entity::get_component(id, selected_entity()) {
                entity::add_component(selected, outline_recursive(), Vec4::ZERO);
            }

            entity::add_component(entity, outline_recursive(), Vec4::ONE);
            entity::add_component(id, selected_entity(), entity);
        }
    });
}
