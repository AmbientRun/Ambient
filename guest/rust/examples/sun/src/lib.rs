use kiwi_api::{
    components::core::{
        app::main_scene,
        camera::{aspect_ratio_from_window, perspective_infinite_reverse},
        game_objects::player_camera,
        primitives::{quad, sphere, sphere_radius},
        rendering::{cast_shadows, color, fog_density, light_diffuse, sky, sun},
        transform::{lookat_center, rotation, scale, translation},
    },
    prelude::*,
};

#[main]
pub async fn main() -> EventResult {
    entity::game_object_base()
        .with_default(player_camera())
        .with(translation(), vec3(5., 5., 2.))
        .with(lookat_center(), vec3(0., 0., 1.))
        .with(perspective_infinite_reverse(), ())
        .with(aspect_ratio_from_window(), ())
        .spawn();

    entity::game_object_base()
        .with_default(quad())
        .with(scale(), Vec3::ONE * 20.)
        .with(color(), vec4(1., 0., 0., 1.))
        .spawn();

    entity::game_object_base().with_default(sky()).spawn();

    entity::game_object_base()
        .with_default(cast_shadows())
        .with_default(sphere())
        .with(sphere_radius(), 1.)
        .with(translation(), vec3(0., 0., 1.))
        .with(color(), vec4(1., 1., 1., 1.))
        .spawn();

    let sun = entity::game_object_base()
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
