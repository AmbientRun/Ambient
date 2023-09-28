use ambient_api::{
    core::{hierarchy::components::parent, transform::components::local_to_parent},
    prelude::*,
};

use packages::{
    gun_laser::concepts::{GunLaser, GunLaserOptional},
    tangent_schema::{concepts::VehicleDef, vehicle::components as vc},
};

const DEF_HOTRELOADING: bool = true;

#[main]
pub fn main() {
    make_allrounder();
    make_speedy();
    make_tank();

    if DEF_HOTRELOADING {
        let defs_query = query(VehicleDef::as_query()).build();
        for (_id, def) in defs_query.evaluate() {
            std::fs::write(
                format!("{}.json", def.name),
                serde_json::to_string_pretty(&def).unwrap(),
            )
            .unwrap();
        }
        fixed_rate_tick(Duration::from_secs(2), move |_| {
            for (id, def) in defs_query.evaluate() {
                let mut new_def = serde_json::from_str::<VehicleDef>(
                    &std::fs::read_to_string(format!("{}.json", def.name)).unwrap(),
                )
                .unwrap();
                new_def.model_url = def.model_url.clone();
                if new_def != def {
                    entity::add_components(id, new_def);
                    println!("Reloaded {}", def.name);
                }
            }
        });
    }
}

fn make_allrounder() {
    let mut def = serde_json::from_str::<VehicleDef>(include_str!("Thunderstrike.json")).unwrap();
    def.model_url = packages::kenney_space_kit::assets::url("craft_speederA.glb/models/main.json");
    let def = def.spawn();

    spawn_query(vc::def_ref())
        .requires(vc::is_vehicle())
        .bind(move |vehicles| {
            for (vehicle_id, def_ref) in vehicles {
                if def_ref != def {
                    continue;
                }

                let weapon_ids = (-1..=1)
                    .step_by(2)
                    .map(|i| {
                        GunLaser {
                            is_gun_laser: (),
                            local_to_world: default(),
                            damage: 20.0,
                            time_between_shots: Duration::from_millis(250),
                            optional: GunLaserOptional {
                                translation: Some(vec3(i as f32 * 0.15, -1.35, 0.0)),
                                rotation: Some(default()),
                                ..default()
                            },
                        }
                        .make()
                        .with(parent(), vehicle_id)
                        .with(local_to_parent(), default())
                        .spawn()
                    })
                    .collect();

                entity::add_component(vehicle_id, vc::aimable_weapon_refs(), weapon_ids);
            }
        });
}

fn make_speedy() {
    let mut def = serde_json::from_str::<VehicleDef>(include_str!("Swiftshadow.json")).unwrap();
    def.model_url = packages::kenney_space_kit::assets::url("craft_racer.glb/models/main.json");
    let def = def.spawn();

    spawn_query(vc::def_ref())
        .requires(vc::is_vehicle())
        .bind(move |vehicles| {
            for (vehicle_id, def_ref) in vehicles {
                if def_ref != def {
                    continue;
                }

                let weapon_id = GunLaser {
                    is_gun_laser: (),
                    local_to_world: default(),
                    damage: 20.0,
                    time_between_shots: Duration::from_millis(500),
                    optional: GunLaserOptional {
                        translation: Some(vec3(0.0, -1.65, 0.3)),
                        rotation: Some(default()),
                        ..default()
                    },
                }
                .make()
                .with(parent(), vehicle_id)
                .with(local_to_parent(), default())
                .spawn();

                entity::add_component(vehicle_id, vc::aimable_weapon_refs(), vec![weapon_id]);
            }
        });
}

fn make_tank() {
    let mut def = serde_json::from_str::<VehicleDef>(include_str!("Ironclad.json")).unwrap();
    def.model_url = packages::kenney_space_kit::assets::url("craft_miner.glb/models/main.json");
    let def = def.spawn();

    spawn_query(vc::def_ref())
        .requires(vc::is_vehicle())
        .bind(move |vehicles| {
            for (vehicle_id, def_ref) in vehicles {
                if def_ref != def {
                    continue;
                }

                let weapon_id = GunLaser {
                    is_gun_laser: (),
                    local_to_world: default(),
                    damage: 60.0,
                    time_between_shots: Duration::from_millis(1250),
                    optional: GunLaserOptional {
                        translation: Some(vec3(0.0, -1.35, 0.15)),
                        rotation: Some(default()),
                        ..default()
                    },
                }
                .make()
                .with(parent(), vehicle_id)
                .with(local_to_parent(), default())
                .spawn();

                entity::add_component(vehicle_id, vc::aimable_weapon_refs(), vec![weapon_id]);
            }
        });
}
