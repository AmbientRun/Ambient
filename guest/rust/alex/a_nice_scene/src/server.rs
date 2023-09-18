use ambient_api::prelude::*;

#[main]
pub fn main() {
    plrs_fps_controlled();
    ground_plane();
    load_scene();
    rising_falling_cube();
}

pub fn rising_falling_cube() {
    use ambient_api::core::{
        physics::components::cube_collider, primitives::components::cube,
        rendering::components::color, transform::components::translation,
    };
    use std::f32::consts::PI;
    let rfc = Entity::new()
        .with(translation(), vec3(10., 10., 0.))
        .with(color(), vec4(1., 0., 0., 1.))
        .with(cube(), ())
        .with(cube_collider(), vec3(1., 1., 1.))
        .spawn();
    ambient_api::core::messages::Frame::subscribe(move |_| {
        let t: f32 = game_time().as_secs_f32();
        entity::set_component(rfc, translation(), vec3(10., 10., t.sin()));
    });
}

pub fn plrs_fps_controlled() {
    use ambient_api::core::{
        model::components::model_from_url,
        player::components::{is_player, user_id},
    };
    use packages::{
        character_animation::components::basic_character_animations,
        fps_controller::components::use_fps_controller,
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
                    .with(basic_character_animations(), plr),
            );
        }
    });
}

pub fn ground_plane() {
    use ambient_api::core::{
        physics::components::plane_collider, primitives::components::quad,
        transform::components::local_to_world,
    };
    Entity::new()
        .with(local_to_world(), default())
        .with(quad(), ())
        .with(plane_collider(), ())
        .spawn();
}

pub fn spawn_sun() -> EntityId {
    use ambient_api::core::{
        app::components::main_scene,
        rendering::components::{
            fog_color, fog_density, fog_height_falloff, light_diffuse, sky, sun,
        },
        transform::components::rotation,
    };

    Entity::new().with(sky(), ()).spawn();

    Entity::new()
        .with(sun(), 0.0)
        .with(rotation(), Default::default())
        .with(main_scene(), ())
        .with(light_diffuse(), Vec3::ONE)
        .with(fog_color(), vec3(0.88, 0.37, 0.34))
        // .with(fog_color(), vec3(0., 0., 0.))
        .with(fog_density(), 0.1)
        .with(fog_height_falloff(), 0.01)
        .with(rotation(), Quat::from_rotation_y(190.0f32.to_radians()))
        .spawn()
}

mod scene_deep_pit;
mod sceneloader;

pub fn load_scene() {
    use ambient_api::core::{
        app::components::name,
        physics::components::cube_collider,
        prefab::components::prefab_from_url,
        primitives::components::cube,
        transform::components::{rotation, scale, translation},
    };

    let nodes = crate::sceneloader::scene_contents_to_nodes(scene_deep_pit::CONTENTS);

    for (_key, node) in nodes {
        let node_pos: Option<Vec3> = node.pos;
        let node_rot: Option<Quat> = node.rot;
        let node_siz: Option<Vec3> = node.siz;
        match node.name.as_str() {
            // "player" => {
            //     player.add_component(rotation(), node_rot.unwrap());
            // },
            "sun" => {
                let sun = spawn_sun();

                // entity::add_component(sun, rotation(), node_rot.unwrap());

                entity::add_component(
                    sun,
                    rotation(),
                    node_rot.unwrap() * Quat::from_rotation_z(3.1416),
                ); // sun reverse rotation
            }
            "cube1" | "cube2" => {
                println!(
                    "Spawn one cube @ pos {:?} rot {:?}",
                    node_pos.unwrap(),
                    node_rot.unwrap()
                );
                Entity::new()
                    .with(cube(), ())
                    .with(cube_collider(), vec3(1., 1., 1.))
                    .with(translation(), node_pos.unwrap())
                    .with(rotation(), node_rot.unwrap())
                    .with(scale(), node_siz.unwrap())
                    .spawn();
            }
            // "camera" => {
            //     println!(
            //         "Yes: Found camera! @ pos {:?} rot {:?}",
            //         node_pos.unwrap(),
            //         node_rot.unwrap()
            //     );
            //     entity::set_component(main_camera_ent, translation(), node_pos.unwrap());
            //     entity::set_component(main_camera_ent, rotation(), node_rot.unwrap());
            // }
            _ => {
                if let Some(path) = node.path {
                    Entity::new()
                        .with(name(), node.name)
                        // .with_default(cube())
                        .with(translation(), node_pos.unwrap())
                        .with(rotation(), node_rot.unwrap())
                        .with(scale(), node_siz.unwrap())
                        .with(prefab_from_url(), crate::packages::this::assets::url(&path))
                        .spawn();
                }
            }
        }
    }
}
