// TODO: this should vary based on the game type

use ambient_api::{
    core::{player::components::player, transform::components::translation},
    prelude::*,
};

use afps::{
    afps_fpsrule::{
        components::{
            heal_timeout, hit_freeze, player_deathcount, player_health, player_killcount,
        },
        messages::Shoot,
    },
    afps_fpsui::components::player_name,
};

#[main]
pub fn main() {
    spawn_query(player()).bind(|results| {
        for (id, ()) in results {
            run_async(async move {
                entity::wait_for_component(id, player_name()).await;
                entity::add_component(id, player_health(), 100);
                entity::add_component(id, hit_freeze(), 0);
                entity::add_component(id, player_killcount(), 0);
                entity::add_component(id, player_deathcount(), 0);
                entity::add_component(id, heal_timeout(), 0);
            });
        }
    });

    Shoot::subscribe(move |_source, msg| {
        let result = physics::raycast_first(msg.ray_origin, msg.ray_dir);

        if let Some(hit) = result {
            if hit.entity == msg.source {
                eprintln!("self hit");
                return;
            }

            if let Some(old_health) = entity::get_component(hit.entity, player_health()) {
                if old_health <= 0 {
                    return;
                }

                let new_health = (old_health - 10).max(0);
                entity::set_component(hit.entity, player_health(), new_health);

                if old_health > 0 && new_health <= 0 {
                    println!("player dead, waiting for respawn");
                    entity::set_component(hit.entity, hit_freeze(), 114);
                    entity::mutate_component(msg.source, player_killcount(), |count| {
                        *count += 1;
                    });
                    entity::mutate_component(hit.entity, player_deathcount(), |count| {
                        *count += 1;
                    });
                    run_async(async move {
                        sleep(114. / 60.).await;

                        if !entity::exists(hit.entity) {
                            return;
                        }

                        entity::set_component(
                            hit.entity,
                            translation(),
                            vec3(random::<f32>() * 10.0, random::<f32>() * 10.0, 2.0),
                        );
                        entity::set_component(hit.entity, player_health(), 100);
                        entity::set_component(hit.entity, hit_freeze(), 0);
                    });
                } else {
                    entity::set_component(hit.entity, hit_freeze(), 20);
                    entity::set_component(hit.entity, heal_timeout(), 150);
                }
            }
        }
    });

    query((player(), heal_timeout())).each_frame(move |entities| {
        for (e, (_, old_timeout)) in entities {
            let new_timeout = old_timeout - 1;
            entity::set_component(e, heal_timeout(), new_timeout);
        }
    });

    let healables = query((player(), player_health())).build();
    run_async(async move {
        loop {
            sleep(1.0).await;

            for (e, (_, old_health)) in healables.evaluate() {
                if let Some(timeout) = entity::get_component(e, heal_timeout()) {
                    if timeout > 0 {
                        continue;
                    }
                }

                let new_health = old_health + 1;
                if new_health <= 100 {
                    entity::set_component(e, player_health(), new_health);
                }
            }
        }
    });
}
