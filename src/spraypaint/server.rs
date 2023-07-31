// use ambient_api::{
//     components::core::{primitives::cube, transform::translation},
//     concepts::make_transformable,
//     physics,
//     prelude::*,
// };
use ambient_api::{
    components::core::{
        player::player,
        prefab::prefab_from_url,
        rendering::decal_from_url,
        transform::{scale, translation},
    },
    concepts::make_transformable,
    prelude::*,
};

#[main]

pub fn main() {
    println!("Spraypaint server started");
    messages::Spraypaint::subscribe(move |source, msg| {
        println!("Spray got");
        if let Some(hit) = physics::raycast_first(msg.origin, msg.dir) {
            // println!("hit {:?}", hit.position);
            let decal_url = asset::url("assets/spray/spray/pipeline.toml/0/mat.json").unwrap();

            Entity::new()
                .with_merge(make_transformable())
                .with(translation(), hit.position)
                .with(decal_from_url(), decal_url)
                .spawn();

            Entity::new()
                .with_merge(make_transformable())
                .with(
                    prefab_from_url(),
                    asset::url("assets/map/claymore.glb").unwrap(),
                )
                .with_default(components::claymore())
                .with(translation(), hit.position + vec3(0., 0., 0.15))
                .with(scale(), Vec3::ONE * 1.0)
                .spawn();
        }
    });

    // claymore
    let player_query = query(translation()).requires(player()).build();
    query((components::claymore(), translation())).each_frame(move |entities| {
        for (e, (_, cm_pos)) in entities {
            // let cm_pos = vec2(cm_pos3.x, cm_pos3.y);
            let players: Vec<(EntityId, Vec3)> = player_query.evaluate();
            for (player, player_pos) in players {
                // let player_pos = vec2(pos.x, pos.y);
                let distance = (cm_pos - player_pos).length();
                if distance < 2. {
                    println!("claymore hit");
                    messages::Explosion::new(cm_pos).send_local_broadcast(false);
                    entity::despawn(e);
                    entity::add_component(player, components::hit_freeze(), 180);
                    entity::set_component(player, components::player_health(), 0);
                    entity::set_component(player, components::player_vspeed(), 0.8);

                    if entity::has_component(
                        entity::synchronized_resources(),
                        components::kill_log(),
                    ) {
                        entity::mutate_component(
                            entity::synchronized_resources(),
                            components::kill_log(),
                            |v| {
                                v.push(format!(
                                    "\u{f119} {} was blown up",
                                    entity::get_component(player, components::player_name())
                                        .unwrap_or("unknown".to_string())
                                ));
                                if v.len() >= 4 {
                                    v.remove(0);
                                }
                            },
                        );
                    } else {
                        entity::add_component(
                            entity::synchronized_resources(),
                            components::kill_log(),
                            vec![format!(
                                "\u{f119} {} was blown up",
                                entity::get_component(player, components::player_name())
                                    .unwrap_or("unknown".to_string())
                            )],
                        );
                    }
                    run_async(async move {
                        sleep(3.).await;
                        entity::set_component(
                            player,
                            translation(),
                            vec3(random::<f32>() * 10.0, random::<f32>() * 60.0 - 30., 2.0),
                        );
                        entity::set_component(player, components::player_health(), 100);
                        entity::set_component(player, components::hit_freeze(), 0);
                    });
                }
            }
        }
    });
}
