use ambient_api::prelude::*;

#[main]
pub fn main() {
    spawn_query(player()).bind(move |players| {
        for (id, _) in players {
            entity::add_component(id, components::player_health(), 100);
            entity::add_component(id, components::hit_freeze(), 0);
        }
    });

    messages::Shoot::subscribe(move |_source, msg| {
        let result = physics::raycast_first(msg.ray_origin, msg.ray_dir);
        // let game_type =
        //     entity::get_component(entity::resources(), components::game_type()).unwrap();
        if let Some(hit) = result {
            if entity::has_component(hit.entity, components::player_health()) {
                let old_health =
                    entity::get_component(hit.entity, components::player_health()).unwrap();
                println!("hit player: {}", old_health);
                if old_health <= 0 {
                    return;
                }
                let new_health = (old_health - 10).max(0);
                entity::set_component(hit.entity, components::player_health(), new_health);
                // let model =
                //     entity::get_component(hit.entity, components::player_model_ref()).unwrap();

                if old_health > 0 && new_health <= 0 {
                    println!("player death");
                    entity::set_component(
                        hit.entity,
                        translation(),
                        vec3(random::<f32>() * 10.0, random::<f32>() * 10.0, 2.0),
                    );
                }
            }
        }
    });
}
