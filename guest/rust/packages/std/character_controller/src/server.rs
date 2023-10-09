use ambient_api::{
    core::{
        app::components::name,
        transform::{
            components::{local_to_parent, rotation, translation},
            concepts::Transformable,
        },
    },
    entity::{add_child, add_component, get_component, set_component},
    prelude::*,
};
use packages::{
    this::{
        components::use_character_controller,
        messages::{Input, Jump},
    },
    unit_schema::components::{
        head_ref, is_on_ground, jumping, run_direction, running, shooting, vertical_velocity,
    },
};
use std::f32::consts::PI;

#[main]
pub fn main() {
    spawn_query(use_character_controller())
        .excludes(head_ref())
        .bind(|players| {
            for (id, _) in players {
                let head = Entity::new()
                    .with(name(), "Head".to_string())
                    .with_merge(Transformable {
                        local_to_world: default(),
                        optional: default(),
                    })
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

        entity::set_component(player_id, run_direction(), msg.run_direction);
        entity::set_component(player_id, running(), msg.running);
        entity::set_component(player_id, shooting(), msg.shooting);
        entity::set_component(player_id, rotation(), Quat::from_rotation_z(msg.body_yaw));
        if let Some(head) = get_component(player_id, head_ref()) {
            set_component(
                head,
                rotation(),
                Quat::from_rotation_y(msg.head_pitch)
                    * Quat::from_rotation_z(PI / 2.)
                    * Quat::from_rotation_x(PI / 2.),
            );
        }
    });

    Jump::subscribe(move |ctx, _msg| {
        let Some(player_id) = ctx.client_entity_id() else {
            return;
        };

        if get_component(player_id, is_on_ground()).unwrap_or_default() {
            entity::set_component(player_id, vertical_velocity(), 0.1);
            entity::set_component(player_id, jumping(), true);
        }
    });
}
