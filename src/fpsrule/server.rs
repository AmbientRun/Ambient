// TODO: add death and kill count

// TODO: this should vary based on the game type

use ambient_api::prelude::*;

#[main]
pub fn main() {
    spawn_query(player()).bind(move |players| {
        for (id, _) in players {
            entity::add_component(id, components::player_health(), 100);
            entity::add_component(id, components::hit_freeze(), 0);
        }
    });

    // TODO: add death and kill count

    // TODO: this should vary based on the game type
    // let game_type =
    //     entity::get_component(entity::resources(), components::game_type()).unwrap();

    messages::Shoot::subscribe(move |_source, msg| {
        let result = physics::raycast_first(msg.ray_origin, msg.ray_dir);

        if let Some(hit) = result {
            if entity::has_component(hit.entity, components::player_health()) {
                let old_health =
                    entity::get_component(hit.entity, components::player_health()).unwrap();
                if old_health <= 0 {
                    return;
                }
                let new_health = (old_health - 10).max(0);
                entity::set_component(hit.entity, components::player_health(), new_health);

                if old_health > 0 && new_health <= 0 {
                    println!("player die, waiting for respawn");
                    entity::set_component(hit.entity, components::hit_freeze(), 114);
                    run_async(async move {
                        sleep(114. / 60.).await;
                        entity::set_component(
                            hit.entity,
                            translation(),
                            vec3(random::<f32>() * 10.0, random::<f32>() * 10.0, 2.0),
                        );
                        entity::set_component(hit.entity, components::player_health(), 100);
                        entity::set_component(hit.entity, components::hit_freeze(), 0);
                    });
                } else {
                    println!("hit player, make the health becomes => {}", new_health);
                    entity::set_component(hit.entity, components::hit_freeze(), 20);
                }
            }
        }
    });
}
