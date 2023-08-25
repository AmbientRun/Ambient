use std::f32::consts::PI;

use ambient_api::{
    core::{
        app::components::{main_scene, name},
        camera::{
            components::{active_camera, aspect_ratio_from_window},
            concepts::make_perspective_infinite_reverse_camera,
        },
        physics::components::dynamic,
        player::components::user_id,
        rendering::components::outline_recursive,
        transform::components::{local_to_world, rotation, translation},
    },
    ecs::GeneralQuery,
    glam::EulerRot,
    once_cell::sync::Lazy,
    prelude::*,
};

use packages::{
    editor_schema::components::in_editor,
    this::{
        components::{
            camera_angle, editor_camera, mouseover_entity, mouseover_position, selected_entity,
        },
        messages::{Input, ToggleEditor},
    },
};

#[main]
pub fn main() {
    ToggleEditor::subscribe(|source, _| {
        let Some(id) = source.client_entity_id() else {
            return;
        };

        let in_editor = entity::mutate_component_with_default(id, in_editor(), true, |in_editor| {
            *in_editor = !*in_editor;
        });

        if in_editor {
            let player_user_id = entity::get_component(id, user_id()).unwrap();

            let old_camera_transform = get_active_camera(&player_user_id)
                .and_then(|camera_id| entity::get_component(camera_id, local_to_world()))
                .map(|transform| transform.to_scale_rotation_translation())
                .map(|(_, r, t)| (r, t));

            let new_camera_position = old_camera_transform.map(|(_, t)| t).unwrap_or_else(|| {
                entity::get_component(id, translation()).unwrap_or_default() + vec3(0.0, 0.0, 5.0)
            });
            let new_camera_angle = old_camera_transform
                .map(|(rot, _)| {
                    let euler = rot.to_euler(EulerRot::ZYX);
                    vec2(euler.0, euler.2)
                })
                .unwrap_or_else(|| vec2(0.0, PI / 2.));

            let camera_id = Entity::new()
                .with_merge(make_perspective_infinite_reverse_camera())
                .with(aspect_ratio_from_window(), EntityId::resources())
                .with(main_scene(), ())
                .with(user_id(), player_user_id)
                .with(translation(), new_camera_position)
                .with(camera_angle(), new_camera_angle)
                .with(name(), "Editor Camera".to_string())
                .with(active_camera(), 10.0)
                .spawn();

            entity::add_component(id, editor_camera(), camera_id);
        } else {
            deselect(id);

            if let Some(camera_id) = entity::get_component(id, editor_camera()) {
                entity::remove_component(id, editor_camera());
                entity::despawn(camera_id);
            }
        }
    });

    Input::subscribe(|source, msg| {
        let Some(id) = source.client_entity_id() else {
            return;
        };
        if !entity::get_component(id, in_editor()).unwrap_or_default() {
            return;
        }

        let Some(camera_id) = entity::get_component(id, editor_camera()) else {
            return;
        };

        let angle = entity::mutate_component_with_default(
            camera_id,
            camera_angle(),
            vec2(0.0, -PI),
            |angle| {
                *angle += msg.aim_delta;
                angle.x %= PI;
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

        if let (Some(mouseover_id), true) =
            (entity::get_component(id, mouseover_entity()), msg.select)
        {
            let previously_selected_id = deselect(id);
            if Some(mouseover_id) != previously_selected_id {
                select(id, mouseover_id);
            }
        }

        if let Some(entity) = entity::get_component(id, selected_entity()) {
            if let Some(translate_to) = msg.translate_to {
                if entity::has_component(entity, translation()) {
                    entity::set_component(entity, translation(), translate_to);
                }
            }

            // TODO: Unfreezing doesn't work because dynamic() isn't updated.
            // With that being said, dynamic() should probably be replaced by a collider_type() enum.
            // Even then, though, that isn't synced back.
            if msg.freeze {
                let is_dynamic = entity::get_component(entity, dynamic()).unwrap_or_default();
                if is_dynamic {
                    physics::freeze(entity)
                } else {
                    physics::unfreeze(entity)
                }
            }
        }
    });
}

// TODO: Move this to ambient_api?
pub fn get_active_camera(player_user_id: &str) -> Option<EntityId> {
    static QUERY: Lazy<GeneralQuery<(Component<f32>, Component<String>)>> =
        Lazy::new(|| query((active_camera(), user_id())).build());

    QUERY
        .evaluate()
        .into_iter()
        .filter(|(_, (_, uid))| *uid == player_user_id)
        .map(|(id, (ordering, _))| (id, ordering))
        .max_by(|x, y| x.1.partial_cmp(&y.1).unwrap_or(std::cmp::Ordering::Less))
        .map(|(id, _)| id)
}

fn deselect(player_id: EntityId) -> Option<EntityId> {
    let Some(selected_id) = entity::get_component(player_id, selected_entity()) else {
        return None;
    };
    entity::remove_component(selected_id, outline_recursive());
    entity::remove_component(player_id, selected_entity());

    Some(selected_id)
}

fn select(player_id: EntityId, entity_id: EntityId) {
    entity::add_component(entity_id, outline_recursive(), Vec4::ONE);
    entity::add_component(player_id, selected_entity(), entity_id);
}
