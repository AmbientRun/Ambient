use ambient_api::{
    core::{
        app::components::name,
        physics::components::plane_collider,
        prefab::components::prefab_from_url,
        primitives::components::{cube, quad},
        rendering::components::{cast_shadows, color},
        transform::components::{rotation, scale, translation},
    },
    prelude::*,
};

#[main]
pub fn main() {
    // ground plane
    Entity::new()
        .with(translation(), Vec3::ZERO)
        .with(quad(), ())
        .with(scale(), Vec3::splat(1000.))
        .with(color(), Vec3::splat(1.).extend(1.)) // purewhite floor
        .with(plane_collider(), ())
        .spawn();

    load_scene();
}

mod sceneloader;

pub fn load_scene() {
    let nodes = crate::sceneloader::scene_contents_to_nodes(include_str!(
        "../scenes/final_storm_scene.tscn"
    ));

    for (_key, node) in nodes {
        if let Some(path) = node.path {
            // println!("Load path {path}");
            if path.ends_with("glb") || path.ends_with("fbx") {
                let ent = Entity::new()
                    .with(name(), node.name.clone())
                    // .with_default(cube())
                    .with(translation(), node.pos.unwrap())
                    .with(rotation(), node.rot.unwrap())
                    .with(scale(), node.siz.unwrap())
                    .with(
                        prefab_from_url(),
                        crate::packages::this::assets::url(("".to_owned() + &path).as_mut_str()),
                    )
                    .with(cast_shadows(), ())
                    .spawn();

                if node.name.contains("fireplace") {
                    println!("Loaded fireplace: '{}'", node.name);
                    entity::add_component(
                        ent,
                        packages::this::components::fireplace_name(),
                        node.name.clone(),
                    );
                }

                match node.name.as_str() {
                    "athena" | "hermaneubis" | "shepherd-boy" => {
                        println!("Loaded statue: '{}'", node.name);
                        entity::add_component(
                            ent,
                            packages::this::components::statue_name(),
                            node.name.clone(),
                        );
                    }
                    _ => {} // do nothing
                }

                // debug pointer for small models
                // Entity::new()
                //     .with(translation(), node.pos.unwrap() + vec3(0., 0., 2.5))
                //     .with(scale(), vec3(0.01, 0.01, 5.0))
                //     .with(cube(), ())
                //     .with(color(), random::<Vec3>().extend(1.))
                //     .spawn();
            }
        }
    }
}
