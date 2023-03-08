use ambient_api::prelude::*;

#[main]
#[cfg(not(feature = "server"))]
pub async fn main() -> EventResult {
    EventOk
}

#[main]
#[cfg(feature = "server")]
pub async fn main() -> EventResult {
    use ambient_api::{
        components::core::{
            app::main_scene,
            camera::aspect_ratio_from_window,
            primitives::{quad, sphere_radius},
            rendering::{cast_shadows, color, fog_density, light_diffuse, sky, sun, water},
            transform::{lookat_center, rotation, scale, translation},
        },
        concepts::{make_perspective_infinite_reverse_camera, make_sphere, make_transformable},
    };

    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with_default(main_scene())
        .with(translation(), vec3(5., 5., 2.))
        .with(lookat_center(), vec3(0., 0., 1.))
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with_default(quad())
        .with(scale(), Vec3::ONE * 20.)
        .with(color(), vec4(1., 0., 0., 1.))
        .with(translation(), vec3(0., 0., 0.01))
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with_default(water())
        .with(scale(), Vec3::ONE * 2000.)
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with_default(sky())
        .spawn();

    Entity::new()
        .with_merge(make_sphere())
        .with_default(cast_shadows())
        .with(sphere_radius(), 1.)
        .with(translation(), vec3(0., 0., 1.))
        .with(color(), vec4(1., 1., 1., 1.))
        .spawn();

    let sun = Entity::new()
        .with_merge(make_transformable())
        .with_default(sun())
        .with_default(rotation())
        .with_default(main_scene())
        .with(light_diffuse(), Vec3::ONE)
        .with(fog_density(), 0.)
        .spawn();

    on(event::FRAME, move |_| {
        let rot = entity::get_component(sun, rotation()).unwrap();
        entity::set_component(sun, rotation(), rot * Quat::from_rotation_y(0.01));
        EventOk
    });

    EventOk
}
