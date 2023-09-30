use std::f32::consts::PI;

use ambient_api::{
    core::{
        app::components::name,
        camera::concepts::{
            PerspectiveInfiniteReverseCamera, PerspectiveInfiniteReverseCameraOptional,
        },
        package::components::main_package_id,
        physics::components::dynamic,
        player::components::user_id,
        rendering::components::outline_recursive,
        transform::components::{rotation, translation},
    },
    glam::EulerRot,
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
    // If the editor is being launched by itself, create a sample scene to edit.
    if entity::get_component(entity::resources(), main_package_id())
        == Some(packages::this::entity())
    {
        make_sample_scene();
    }

    ToggleEditor::subscribe(|ctx, msg| {
        let Some(id) = ctx.client_entity_id() else {
            return;
        };

        let in_editor = entity::mutate_component_with_default(id, in_editor(), true, |in_editor| {
            *in_editor = !*in_editor;
        });

        if in_editor {
            let player_user_id = entity::get_component(id, user_id()).unwrap();

            let old_camera_transform = msg
                .camera_transform
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

            let camera_id = PerspectiveInfiniteReverseCamera {
                optional: PerspectiveInfiniteReverseCameraOptional {
                    translation: Some(new_camera_position),
                    rotation: Some(default()),
                    main_scene: Some(()),
                    aspect_ratio_from_window: Some(entity::resources()),
                    user_id: Some(player_user_id),
                    ..default()
                },
                ..PerspectiveInfiniteReverseCamera::suggested()
            }
            .make()
            .with(camera_angle(), new_camera_angle)
            .with(name(), "Editor Camera".to_string())
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

    Input::subscribe(|ctx, msg| {
        let Some(id) = ctx.client_entity_id() else {
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

fn make_sample_scene() {
    use ambient_api::core::{
        app::components::main_scene,
        package::components::enabled,
        physics::components::{
            angular_velocity, cube_collider, linear_velocity, physics_controlled, plane_collider,
        },
        primitives::components::{cube, quad},
        rendering::components::{cast_shadows, color, fog_density, light_diffuse, sky, sun},
        transform::components::scale,
    };

    entity::set_component(packages::hide_cursor::entity(), enabled(), true);

    entity::add_component(
        entity::synchronized_resources(),
        packages::this::components::has_sample_scene(),
        (),
    );

    // Make sky
    Entity::new().with(sky(), ()).spawn();

    // Make sun
    Entity::new()
        .with(sun(), 0.0)
        .with(rotation(), Quat::from_rotation_y(-45_f32.to_radians()))
        .with(main_scene(), ())
        .with(light_diffuse(), Vec3::ONE)
        .with(fog_density(), 0.)
        .with(main_scene(), ())
        .spawn();

    // Make ground
    Entity::new()
        .with(quad(), ())
        .with(physics_controlled(), ())
        .with(plane_collider(), ())
        .with(dynamic(), false)
        .with(scale(), Vec3::ONE * 4000.)
        .with(color(), Vec4::ONE)
        .spawn();

    // Make boxes
    const Y_SIZE: i32 = 9;
    const X_SIZE: i32 = 9;
    for y in 0..Y_SIZE {
        for x in 0..X_SIZE {
            let position = vec3(
                x as f32 - X_SIZE as f32 / 2.0,
                y as f32 - Y_SIZE as f32 / 2.0,
                1.,
            );

            let s = x as f32 / (X_SIZE - 1) as f32;
            let t = y as f32 / (Y_SIZE - 1) as f32;

            let color_x = vec4(1.0, 0.0, 0.0, 1.0).lerp(vec4(0.0, 1.0, 0.0, 1.0), s);
            let color_y = vec4(0.0, 1.0, 0.0, 1.0).lerp(vec4(0.0, 0.0, 1.0, 1.0), t);

            let new_color = color_x.lerp(color_y, 0.5);

            Entity::new()
                .with(cube(), ())
                .with(physics_controlled(), ())
                .with(cast_shadows(), ())
                .with(linear_velocity(), Vec3::ZERO)
                .with(angular_velocity(), Vec3::ZERO)
                .with(cube_collider(), Vec3::ONE)
                .with(dynamic(), true)
                .with(translation(), position)
                .with(rotation(), Quat::IDENTITY)
                .with(scale(), vec3(0.5, 0.5, 0.5))
                .with(color(), new_color)
                .spawn();
        }
    }
}
