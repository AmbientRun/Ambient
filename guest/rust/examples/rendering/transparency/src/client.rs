use ambient_api::{
    core::{
        app::components::main_scene,
        camera::{
            components::aspect_ratio_from_window, concepts::make_PerspectiveInfiniteReverseCamera,
        },
        primitives::components::{cube, quad},
        rendering::components::{color, transparency_group},
        transform::{
            components::{lookat_target, scale, translation},
            concepts::make_Transformable,
        },
    },
    prelude::*,
};

#[main]
fn main() {
    Entity::new()
        .with_merge(make_PerspectiveInfiniteReverseCamera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with(main_scene(), ())
        .with(translation(), vec3(5., 5., 4.))
        .with(lookat_target(), vec3(0., 0., 2.))
        .spawn();

    Entity::new()
        .with_merge(make_Transformable())
        .with(quad(), ())
        .with(scale(), Vec3::ONE * 10.)
        .with(color(), vec4(1., 0., 0., 1.))
        .spawn();

    Entity::new()
        .with_merge(make_Transformable())
        .with(cube(), ())
        .with(translation(), vec3(0., 0., 1.))
        .with(scale(), Vec3::ONE * 2.)
        .with(color(), vec4(0., 1., 0., 0.5))
        .with(transparency_group(), 0)
        .spawn();
}
