use ambient_api::prelude::*;

#[main]
pub fn main() {
    plrs_fps_controlled();
    ground_plane();
    central_campfire();
    plrs_get_cold();
    load_scene();
}

const CAMPFIRE_POS: Vec3 = Vec3::new(3., 0., 0.);

pub fn plrs_fps_controlled() {
    use ambient_api::core::{
        app::components::name,
        model::components::model_from_url,
        player::components::{is_player, user_id},
        transform::{
            components::{rotation, translation},
            concepts::make_transformable,
        },
    };
    use packages::{
        character_animation::components::basic_character_animations,
        fps_controller::components::use_fps_controller,
        this::components::{coldness, effect_respawn},
    };
    spawn_query((is_player(), user_id())).bind(|plrs| {
        for (plr, (_, uid)) in plrs {
            entity::add_components(
                plr,
                Entity::new()
                    .with(use_fps_controller(), ())
                    .with(
                        model_from_url(),
                        packages::base_assets::assets::url("Y Bot.fbx"),
                    )
                    .with(basic_character_animations(), plr)
                    .with(effect_respawn(), true)
                    .with(coldness(), 0.0),
            );
        }
    });
    spawn_query(effect_respawn()).bind(|plrs| {
        for (plr, _) in plrs {
            let rot = Quat::from_rotation_z(random::<f32>() * 6.28);

            entity::add_component(plr, translation(), rot * vec3(3., 0., 0.) + CAMPFIRE_POS);
            entity::add_component(plr, coldness(), 0.00); // reset coldness.

            // TODO: do 'respawning' animation. getting up off the ground.

            entity::remove_component(plr, effect_respawn());
        }
    });
}

pub fn ground_plane() {
    use ambient_api::core::{
        physics::components::plane_collider,
        primitives::components::quad,
        transform::{components::scale, concepts::make_transformable},
    };
    Entity::new()
        .with_merge(make_transformable())
        .with(quad(), ())
        .with(scale(), Vec3::splat(1000.))
        .with(plane_collider(), ())
        .spawn();
}

const FREEZE_RATE: f32 = 0.05;
const THAW_RATE: f32 = 0.25;
const CAMPFIRE_RANGE: f32 = 10.;

pub fn plrs_get_cold() {
    use ambient_api::core::transform::components::translation;
    use packages::this::components::{coldness, effect_respawn, warmth_radius};

    let find_warmth_sources = query((translation(), warmth_radius())).build();

    query((translation(), coldness())).each_frame(move |plrs| {
        let warmth_sources = find_warmth_sources.evaluate();
        for (plr, (pos, cold)) in plrs {
            let mut warmth: f32 = 0.0;
            for (campfire, (firepos, firerad)) in warmth_sources.iter() {
                warmth = warmth.max(1.0 - pos.distance(*firepos) / firerad);
            }

            if warmth < 0.05 {
                entity::mutate_component(plr, coldness(), |cold| {
                    *cold = (*cold + FREEZE_RATE * delta_time()).min(1.)
                });
            } else {
                entity::mutate_component(plr, coldness(), |cold| {
                    *cold = (*cold - THAW_RATE * delta_time()).max(0.)
                });
            }
            if cold >= 1. {
                entity::add_component(plr, effect_respawn(), true);
            }
        }
    });
}

pub fn central_campfire() {
    use ambient_api::core::{
        model::components::model_from_url, //primitives::concepts::make_sphere,
        rendering::components::outline,
        transform::{
            components::{rotation, translation},
            concepts::make_transformable,
        },
    };
    use packages::this::components::warmth_radius;
    Entity::new()
        .with_merge(make_transformable())
        .with(translation(), CAMPFIRE_POS)
        .with(rotation(), Quat::from_rotation_z(3.00))
        .with(warmth_radius(), CAMPFIRE_RANGE)
        // .with_merge(make_sphere())
        .with(
            model_from_url(),
            packages::this::assets::url("emissive_campfire.glb"),
        )
        .spawn();
}

mod scene_snowstorm_maze;
mod sceneloader;

pub fn load_scene() {
    use ambient_api::core::{
        app::components::name,
        physics::components::cube_collider,
        prefab::components::prefab_from_url,
        primitives::components::cube,
        transform::{
            components::{rotation, scale, translation},
            concepts::make_transformable,
        },
    };

    let nodes = crate::sceneloader::scene_contents_to_nodes(scene_snowstorm_maze::CONTENTS);

    for (_key, node) in nodes {
        if let Some(path) = node.path {
            Entity::new()
                .with(name(), node.name)
                .with_merge(make_transformable())
                // .with_default(cube())
                .with(translation(), node.pos.unwrap())
                .with(rotation(), node.rot.unwrap())
                .with(scale(), node.siz.unwrap())
                .with(prefab_from_url(), crate::packages::this::assets::url(&path))
                .spawn();
        }
    }
}
