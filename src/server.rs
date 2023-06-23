use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        prefab::prefab_from_url,
        primitives::quad,
        transform::{lookat_target, translation},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    prelude::*,
};

#[main]
pub fn main() {
    // let x = Entity::new()
    //     .with_merge(make_transformable())
    //     // .with_merge(make_sphere())
    //     .with(
    //         prefab_from_url(),
    //         asset::url("assets/gun/m4a1_carbine.glb").unwrap(),
    //     )
    //     .with(scale(), vec3(0.3, 0.3, 0.3) * 0.1)
    //     // .with(color(), vec4(1.0, 1.0, 0.0, 1.0))
    //     .with_default(local_to_parent())
    //     // .with_default(reset_scale())
    //     .spawn();
}
