use ambient_api::{
    core::{
        app::components::name,
        physics::concepts::make_character_controller,
        player::components::is_player,
        transform::{
            components::{local_to_parent, rotation, translation},
            concepts::make_transformable,
        },
    },
    entity::{add_child, add_component, get_component},
    prelude::*,
};
use packages::{
    this::{components::use_fps_controller, messages::Input},
    unit_schema::components::{
        head_ref, jumping, run_direction, run_speed_multiplier, running, speed,
        strafe_speed_multiplier, vertical_velocity,
    },
};
use std::f32::consts::PI;

#[main]
pub fn main() {
    spawn_query(use_fps_controller()).bind(move |players| {
        for (id, _) in players {
            entity::add_components(
                id,
                Entity::new()
                    .with_merge(make_character_controller())
                    .with(run_direction(), Vec2::ZERO)
                    .with(vertical_velocity(), 0.)
                    .with(running(), false),
                // .with(player_mouse_delta_x(), 0.0)
                // .with(player_scroll(), 0.0)
                // .with(camera_follow_distance(), 4.0),
            );
        }
    });
    spawn_query(use_fps_controller())
        .excludes(head_ref())
        .bind(|players| {
            for (id, _) in players {
                let head = Entity::new()
                    .with(name(), "Head".to_string())
                    .with_merge(make_transformable())
                    .with(local_to_parent(), Default::default())
                    .with(translation(), Vec3::Z * 2.)
                    .with(
                        rotation(),
                        Quat::from_rotation_z(PI / 2.) * Quat::from_rotation_x(PI / 2.),
                    )
                    .spawn();
                add_child(id, head);
                add_component(id, head_ref(), head);
            }
        });

    Input::subscribe(move |ctx, msg| {
        let Some(player_id) = ctx.client_entity_id() else {
            return;
        };

        entity::set_component(player_id, run_direction(), msg.direction);
        entity::set_component(player_id, rotation(), Quat::from_rotation_z(msg.rotation_z));
        // entity::set_component(player_id, player_scroll(), msg.scroll);

        // entity::mutate_component(player_id, rotation(), |x| {
        //     *x *= Quat::from_rotation_z(msg.mouse_delta_x * 0.01)
        // })
        // .unwrap_or_default();
    });
}
