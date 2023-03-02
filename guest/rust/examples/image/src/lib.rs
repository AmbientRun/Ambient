use ambient_api::{
    components::core::{
        game_objects::player_camera,
        primitives::{cube, quad, sphere_radius, sphere_sectors, sphere_stacks},
        rendering::{color, pbr_material_from_url},
        transform::{lookat_center, scale, translation},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_sphere, make_transformable},
    prelude::*,
};

#[main]
pub async fn main() -> EventResult {
    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with_default(player_camera())
        .with(translation(), vec3(5., 5., 6.))
        .with(lookat_center(), vec3(0., 0., 2.))
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with_default(quad())
        .with(scale(), Vec3::ONE * 10.)
        .with(
            pbr_material_from_url(),
            asset_url("assets/pipeline.json/0/mat.json").unwrap(),
        )
        .spawn();

    EventOk
}
