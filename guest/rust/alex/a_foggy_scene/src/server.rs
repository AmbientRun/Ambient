use ambient_api::{
    core::{
        model::components::model_from_url,
        physics::components::{cube_collider, plane_collider, sphere_collider},
        player::components::is_player,
        primitives::{
            components::{cube, quad},
            concepts::Sphere,
        },
        rendering::components::{cast_shadows, color, outline},
        // rendering::components::{outline, overlay},
        transform::components::{rotation, scale, translation},
    },
    prelude::*,
};
use packages::{
    character_animation::components::basic_character_animations,
    fps_controller::components::use_fps_controller,
    temperature::components::{temperature, temperature_src_radius, temperature_src_rate},
    this::components::ambient_loop,
};

#[main]
pub fn main() {
    entity::add_component(
        packages::package_manager::entity(),
        packages::package_manager::components::mod_manager_for(),
        packages::this::entity(),
    );

    spawn_query(is_player()).bind(|plrs| {
        for (plr, _) in plrs {
            entity::add_components(
                plr,
                Entity::new()
                    .with(use_fps_controller(), ())
                    .with(
                        model_from_url(),
                        packages::this::assets::url("muscle/Muscle Chicken.fbx"),
                    )
                    .with(basic_character_animations(), plr)
                    .with(temperature(), 37.0)
                    .with(temperature_src_rate(), 1.0)
                    .with(temperature_src_radius(), 8.0),
            );
        }
    });

    query(temperature())
        .requires(is_player())
        .each_frame(|plrs| {
            for (plr, temp) in plrs {
                if temp < 21. {
                    // death by freezing - reset to start
                    entity::add_component(plr, translation(), Vec3::ZERO);
                    entity::set_component(plr, temperature(), 37.);
                }
                if temp > 37. {
                    // max body temp
                    entity::set_component(plr, temperature(), 37.);
                }
            }
        });

    Entity::new()
        .with(translation(), Vec3::ZERO)
        .with(temperature_src_rate(), -2.2)
        .with(temperature_src_radius(), core::f32::MAX)
        .spawn();

    Entity::new()
        .with(translation(), vec3(3., 0., 0.25))
        .with(temperature_src_rate(), 5.0) // very warm very fast
        .with(temperature_src_radius(), 20.0) // big radius
        .with(
            model_from_url(),
            packages::this::assets::url("nocoll/pure_white_cone.glb"),
        )
        .with(
            ambient_loop(),
            packages::this::assets::url("4211__dobroide__firecrackling.ogg"),
        )
        .spawn();

    load_scene();
}

mod sceneloader;

pub fn load_scene() {
    use ambient_api::core::{
        app::components::name,
        prefab::components::prefab_from_url,
        // physics::components::cube_collider,
        // primitives::components::cube,
    };

    // we can include the fake default floor in here for now :)
    Entity::new()
        .with(translation(), Vec3::ZERO)
        .with(quad(), ())
        .with(scale(), Vec3::splat(1000.))
        .with(color(), Vec3::splat(0.65).extend(1.))
        .with(plane_collider(), ())
        .spawn();

    let nodes = crate::sceneloader::scene_contents_to_nodes(include_str!(
        "../scenes/snowstorm_mounds.tscn"
    ));

    for (_key, node) in nodes {
        if node.name.starts_with("sphere") {
            Entity::new()
                .with_merge(
                    Sphere {
                        ..Sphere::suggested()
                    }
                    .make(),
                )
                .with(sphere_collider(), 0.5)
                .with(translation(), node.pos.unwrap())
                .with(rotation(), node.rot.unwrap())
                .with(scale(), node.siz.unwrap())
                .with(color(), Vec3::splat(0.65).extend(1.))
                .with(cast_shadows(), ())
                .spawn();
        } else if node.name.starts_with("cube") {
            Entity::new()
                .with(cube(), ())
                .with(cube_collider(), Vec3::splat(1.))
                .with(translation(), node.pos.unwrap())
                .with(rotation(), node.rot.unwrap())
                .with(scale(), node.siz.unwrap())
                .spawn();
        } else if node.name.starts_with("sun") {
            // dbg!(node.rot.unwrap());
        } else if let Some(path) = node.path {
            if path.ends_with("glb") {
                Entity::new()
                    .with(name(), node.name)
                    // .with_default(cube())
                    .with(translation(), node.pos.unwrap())
                    .with(rotation(), node.rot.unwrap())
                    .with(scale(), node.siz.unwrap())
                    .with(prefab_from_url(), crate::packages::this::assets::url(&path))
                    .with(cast_shadows(), ())
                    .spawn();
            }
        }
    }
}
