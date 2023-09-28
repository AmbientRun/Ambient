use ambient_api::{
    core::{
        app::components::main_scene,
        ecs::components::remove_at_game_time,
        hierarchy::components::parent,
        model::components::model_from_url,
        rect::components::{line_from, line_to, line_width},
        rendering::components::{cast_shadows, color, double_sided},
    },
    prelude::*,
};
use packages::{
    explosion::concepts::Explosion,
    tangent_schema::weapon::components::fire,
    this::{
        components::{is_gun_laser, last_shot_time},
        concepts::GunLaser,
        messages::Fire,
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

    query(fire())
        .requires(is_gun_laser())
        .each_frame(|weapons| {
            for (weapon_id, fire) in weapons {
                if !fire {
                    continue;
                }

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

                let parent = entity::get_component(weapon_id, parent());

                if optional
                    .last_shot_time
                    .is_some_and(|lst| game_time() < lst + time_between_shots)
                {
                    return;
                }

                Fire { weapon_id }.send_client_broadcast_unreliable();

                let p0 = local_to_world.transform_point3(vec3(0.0, -0.5, 0.1));
                let dir = local_to_world.transform_vector3(-Vec3::Y);
                let p1 = physics::raycast(p0, dir)
                    .into_iter()
                    .find(|h| h.entity != weapon_id && Some(h.entity) != parent)
                    .map(|h| h.position)
                    .unwrap_or_else(|| p0 + dir * 1_000_000.0);

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
