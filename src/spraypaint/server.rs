// use ambient_api::{
//     components::core::{primitives::cube, transform::translation},
//     concepts::make_transformable,
//     physics,
//     prelude::*,
// };
use ambient_api::{
    components::core::{player::player, rendering::decal_from_url, transform::translation},
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
                .with_default(components::claymore())
                .with(decal_from_url(), decal_url)
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
                    entity::set_component(player, components::hit_freeze(), 180);
                    entity::set_component(player, components::player_health(), 0);
                    entity::set_component(player, components::player_vspeed(), 0.9);
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
