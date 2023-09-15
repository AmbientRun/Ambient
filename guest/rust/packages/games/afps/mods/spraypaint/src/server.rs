// use ambient_api::{
//     components::core::{primitives::cube, transform::translation},
//     concepts::make_Transformable,
//     physics,
//     prelude::*,
// };
use ambient_api::{
    core::{
        player::components::is_player,
        prefab::components::prefab_from_url,
        primitives::components::{cube, quad},
        rendering::components::decal_from_url,
        transform::components::{scale, translation},
    },
    prelude::*,
};

use crate::packages::unit_schema::components::{health, vertical_velocity};
use packages::afps_schema::components;
use packages::{
    afps_schema::messages::Explosion,
    this::{assets, components::claymore, messages::Spraypaint},
};

#[main]

pub fn main() {
    Spraypaint::subscribe(move |_ctx, msg| {
        if let Some(hit) = physics::raycast_first(msg.origin, msg.dir) {
            let player_pos = entity::get_component(msg.source, translation()).unwrap();
            let distance = (player_pos - hit.position).length();
            if distance > 12. {
                // too far
                return;
            }

            if !entity::has_component(hit.entity, cube())
                && !entity::has_component(hit.entity, quad())
            {
                println!("not a valid surface");
                return;
            }

            let decal_url = assets::url("pipeline.toml/0/mat.json");

            Entity::new()
                .with(translation(), hit.position)
                .with(decal_from_url(), decal_url)
                .spawn();

            Entity::new()
                .with(prefab_from_url(), assets::url("claymore.glb"))
                .with(claymore(), msg.source)
                .with(translation(), hit.position + vec3(0., 0., 0.15))
                .with(scale(), Vec3::ONE * 1.0)
                .spawn();
        }
    });

    // claymore
    let player_query = query(translation()).requires(is_player()).build();
    query((claymore(), translation())).each_frame(move |entities| {
        for (e, (source_id, cm_pos)) in entities {
            let source = entity::get_component(source_id, components::player_name())
                .unwrap_or("unknown".to_string());
            let players: Vec<(EntityId, Vec3)> = player_query.evaluate();
            for (player, player_pos) in players {
                // let player_pos = vec2(pos.x, pos.y);
                let distance = (cm_pos - player_pos).length();
                if distance < 2. {
                    println!("claymore hit");
                    Explosion::new(cm_pos).send_local_broadcast(false);
                    entity::despawn(e);
                    entity::add_component(player, components::hit_freeze(), 180);
                    entity::set_component(player, health(), 0.);
                    entity::set_component(player, vertical_velocity(), 0.8);

                    entity::mutate_component(source_id, components::player_killcount(), |count| {
                        *count += 1;
                    });
                    entity::mutate_component(player, components::player_deathcount(), |count| {
                        *count += 1;
                    });

                    if entity::has_component(
                        entity::synchronized_resources(),
                        components::kill_log(),
                    ) {
                        entity::mutate_component(
                            entity::synchronized_resources(),
                            components::kill_log(),
                            |v| {
                                v.push(format!(
                                    // "\u{f119} {} was blown up by \u{f118} {}",
                                    "[{}] \u{f1e2} \u{f061} [{}]",
                                    source,
                                    entity::get_component(player, components::player_name())
                                        .unwrap_or("unknown".to_string()),
                                ));
                            },
                        );
                        remove_last_history();
                    } else {
                        entity::add_component(
                            entity::synchronized_resources(),
                            components::kill_log(),
                            vec![format!(
                                // "\u{f119} {} was blown up by \u{f118} {}",
                                "[{}] \u{f1e2} \u{f061} [{}]",
                                source,
                                entity::get_component(player, components::player_name())
                                    .unwrap_or("unknown".to_string()),
                            )],
                        );
                        remove_last_history();
                    }
                    run_async(async move {
                        sleep(3.).await;
                        entity::set_component(
                            player,
                            translation(),
                            vec3(random::<f32>() * 10.0, random::<f32>() * 60.0 - 30., 2.0),
                        );
                        entity::set_component(player, health(), 100.);
                        entity::set_component(player, components::hit_freeze(), 0);
                    });
                }
            }
        }
    });
}

fn remove_last_history() {
    run_async(async move {
        sleep(10.0).await;
        entity::mutate_component(
            entity::synchronized_resources(),
            components::kill_log(),
            |v| {
                if !v.is_empty() {
                    v.remove(0);
                }
            },
        );
    });
}
