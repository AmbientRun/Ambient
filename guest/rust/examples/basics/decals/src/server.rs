use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        primitives::{quad, cube},
        rendering::{color, transparency_group},
        transform::{lookat_target, rotation, scale, translation},
    },
    concepts::{make_perspective_infinite_reverse_camera},
    prelude::*,
};

use core::f32::consts::PI;

// from discussion at
// https://discord.com/channels/894505972289134632/1078283561540530216/1096581219925377195

#[main]
pub fn main() {
    Entity::new()
        .with_default(cube())
        .with(translation(), Vec3::Z)
        .with(color(), vec4(0.5, 0.5, 0.5, 1.))
        .with_default(cast_shadows())
        .spawn();

    Entity::new()
        .with_default(quad())
        .with(scale(), Vec3::ONE * 10.)
        .spawn();

    Entity::new()
        .with_default(cube())
        .with(scale(), vec3(2., 2., 4.))
        .with(rotation(), Quat::from_rotation_y(PI / 4.) * Quat::from_rotation_z(PI / 4.))
        .with(
            decal_from_url(),
            asset::url("assets/pipeline.json/0/mat.json").unwrap(),
        )
        .spawn();

     Entity::new()
        .with_default(cube())
        .with(scale(), vec3(2., 2., 4.))
        .with(rotation(), Quat::from_rotation_y(PI / 4.) * Quat::from_rotation_z(PI / 4.))
        .with(color(), vec4(0., 1., 0., 0.5))
        .with(transparency_group(), 0)
        .spawn();

    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with(translation(), vec3(5., 5., 6.))
        .with(lookat_target(), vec3(0., 0., 2.))
        .with_default(main_scene())
        .spawn();
}
