use ambient_api::{
    core::{
        app::components::main_scene,
        camera::{
            components::aspect_ratio_from_window, concepts::make_PerspectiveInfiniteReverseCamera,
        },
        messages::Frame,
        primitives::{
            components::{quad, sphere_radius},
            concepts::make_Sphere,
        },
        rendering::components::{cast_shadows, color, fog_density, light_diffuse, sky, sun, water},
        transform::{
            components::{lookat_target, rotation, scale, translation},
            concepts::make_Transformable,
        },
    },
    prelude::*,
};

#[main]
pub fn main() {
    Entity::new()
        .with_merge(make_PerspectiveInfiniteReverseCamera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with(main_scene(), ())
        .with(translation(), vec3(5., 5., 2.))
        .with(lookat_target(), vec3(0., 0., 1.))
        .spawn();

    Entity::new()
        .with_merge(make_Transformable())
        .with(quad(), ())
        .with(scale(), Vec3::ONE * 20.)
        .with(color(), vec4(1., 0., 0., 1.))
        .with(translation(), vec3(0., 0., 0.01))
        .spawn();

    Entity::new()
        .with_merge(make_Transformable())
        .with(water(), ())
        .with(scale(), Vec3::ONE * 2000.)
        .spawn();

    Entity::new()
        .with_merge(make_Transformable())
        .with(sky(), ())
        .spawn();

    Entity::new()
        .with_merge(make_Sphere())
        .with(cast_shadows(), ())
        .with(sphere_radius(), 1.)
        .with(translation(), vec3(0., 0., 1.))
        .with(color(), vec4(1., 1., 1., 1.))
        .spawn();

    let sun = Entity::new()
        .with_merge(make_Transformable())
        .with(sun(), 0.0)
        .with(rotation(), Quat::IDENTITY)
        .with(main_scene(), ())
        .with(light_diffuse(), Vec3::ONE)
        .with(fog_density(), 0.)
        .spawn();

    Frame::subscribe(move |_| {
        let rot = entity::get_component(sun, rotation()).unwrap();
        entity::set_component(sun, rotation(), rot * Quat::from_rotation_y(0.01));
    });
}
