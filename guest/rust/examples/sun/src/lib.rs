use kiwi_api::{
    components::core::{
        app::main_scene,
        camera::{aspect_ratio_from_window, perspective_infinite_reverse},
        game_objects::player_camera,
        primitives::{quad, sphere_radius, sphere_sectors, sphere_stacks},
        rendering::{cast_shadows, color, sun},
        transform::{lookat_center, rotation, scale, translation},
    },
    prelude::*,
};

#[main]
pub async fn main() -> EventResult {
    entity::game_object_base()
        .with_default(player_camera())
        .with(translation(), vec3(5., 5., 4.))
        .with(lookat_center(), vec3(0., 0., 0.))
        .with(perspective_infinite_reverse(), ())
        .with(aspect_ratio_from_window(), ())
        .spawn();

    entity::game_object_base()
        .with_default(quad())
        .with(scale(), Vec3::ONE * 10.)
        .with(color(), vec4(1., 0., 0., 1.))
        .spawn();

    entity::game_object_base()
        .with(translation(), vec3(0., 0., 0.))
        .with(sphere_radius(), 1.)
        .with(sphere_sectors(), 36)
        .with(sphere_stacks(), 18)
        .with(color(), vec4(1., 1., 1., 1.))
        .with_default(cast_shadows())
        .spawn();

    let sun = entity::game_object_base()
        .with_default(sun())
        .with_default(rotation())
        .with_default(main_scene())
        .spawn();

    on(event::FRAME, move |_| {
        let rot = entity::get_component(sun, rotation()).unwrap();
        entity::set_component(sun, rotation(), rot * Quat::from_rotation_y(0.01));
        EventOk
    });

    EventOk
}
