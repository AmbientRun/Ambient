use ambient_api::{
    core::{
        app::components::main_scene,
        ecs::components::remove_at_game_time,
        model::components::model_from_url,
        rect::components::{line_from, line_to, line_width},
        rendering::components::{cast_shadows, color, double_sided},
    },
    prelude::*,
};
use packages::{
    tangent_schema::{concepts::Explosion, weapon::messages::Fire},
    this::{
        components::{is_gun_laser, last_shot_time},
        concepts::GunLaser,
    },
};

#[main]
pub fn main() {
    spawn_query(is_gun_laser()).bind(|guns| {
        for (weapon_id, _) in guns {
            entity::add_components(
                weapon_id,
                Entity::new()
                    .with(cast_shadows(), ())
                    .with(
                        model_from_url(),
                        packages::kenney_space_kit::assets::url("weapon_gun.glb/models/main.json"),
                    )
                    .with(last_shot_time(), game_time()),
            );
        }
    });

    Fire::subscribe(|ctx, msg| {
        if ctx.local().is_none() {
            return;
        }

        let weapon_id = msg.weapon_id;
        let Some(GunLaser {
            local_to_world,
            is_gun_laser: _,
            damage,
            time_between_shots,
            optional,
        }) = GunLaser::get_spawned(weapon_id)
        else {
            return;
        };

        let Some(last_shot_time) = optional.last_shot_time else {
            return;
        };

        if game_time() < last_shot_time + time_between_shots {
            return;
        }

        msg.send_client_broadcast_unreliable();

        let p0 = local_to_world.transform_point3(vec3(0.0, -0.1, 0.1));
        if let Some(hit) = physics::raycast_first(p0, local_to_world.transform_vector3(-Vec3::Y)) {
            let p1 = hit.position;

            Entity::new()
                .with(main_scene(), ())
                .with(line_from(), p0)
                .with(line_to(), p1)
                .with(line_width(), 0.2)
                .with(color(), vec4(0.8, 0.3, 0.0, 1.0))
                .with(double_sided(), true)
                .with(
                    remove_at_game_time(),
                    game_time() + Duration::from_millis(100),
                )
                .spawn();

            Explosion {
                is_explosion: (),
                radius: 1.0,
                damage,
                translation: p1,
                optional: default(),
            }
            .spawn();

            entity::add_component(weapon_id, self::last_shot_time(), game_time());
        }
    });
}
