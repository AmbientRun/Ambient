use ambient_api::{
    core::{
        app::components::name,
        physics::components::plane_collider,
        prefab::components::prefab_from_url,
        primitives::components::quad,
        rendering::components::{cast_shadows, color},
        transform::components::{rotation, scale, translation},
    },
    prelude::*,
};

#[main]
pub fn main() {
    load_scene();
}

mod sceneloader;

pub fn load_scene() {
    let nodes = crate::sceneloader::scene_contents_to_nodes(include_str!(
        "../scenes/final_storm_scene.tscn"
    ));

    for (_key, node) in nodes {
        if let Some(path) = node.path {
            if path.ends_with("glb") || path.ends_with("fbx") {
                Entity::new()
                    .with(name(), node.name)
                    // .with_default(cube())
                    .with(translation(), node.pos.unwrap())
                    .with(rotation(), node.rot.unwrap())
                    .with(scale(), node.siz.unwrap())
                    .with(
                        prefab_from_url(),
                        crate::packages::this::assets::url(
                            ("scene/".to_owned() + &path).as_mut_str(),
                        ),
                    )
                    .with(cast_shadows(), ())
                    .spawn();
            }
        }
    }

    // ground plane
    Entity::new()
        .with(translation(), Vec3::ZERO)
        .with(quad(), ())
        .with(scale(), Vec3::splat(1000.))
        .with(color(), Vec3::splat(1.).extend(1.)) // purewhite floor
        .with(plane_collider(), ())
        .spawn();
}
