use std::{f32::consts::PI, time::Instant};

use glam::*;
use kiwi_app::{App, AppBuilder};
use kiwi_core::{
    camera::active_camera,
    main_scene,
    transform::{rotation, translation},
};
use kiwi_ecs::{query_mut, DynSystem, EntityData};
use kiwi_element::ElementComponentExt;
use kiwi_renderer::{light_ambient, light_diffuse, sun};
use kiwi_sky::{self, Clouds};
use kiwi_std::math::SphericalCoords;
use tokio::runtime::Handle;

fn day_night_cycle() -> DynSystem {
    let start = Instant::now();
    query_mut((rotation(),), (sun(),)).to_system(move |q, world, state, _| {
        let day = start.elapsed().as_secs_f32() / 5.0;

        let n_rot = Quat::from_axis_angle(vec3(1.0, 0.0, 0.5).normalize(), day);

        for (_, (rot,), _) in q.iter(world, state) {
            *rot = n_rot;
        }
    })
}

fn init(app: &mut App, _: Handle) {
    app.add_system(day_night_cycle());
    app.add_system(Box::new(kiwi_sky::clouds_system()));

    let world = &mut app.world;

    Clouds {}.el().spawn_interactive(world);
    // ElementRenderer::new(world, Group(vec![Clouds { cloud_count: 30 }.el()]).el());

    kiwi_cameras::spherical::new(vec3(0., 0., 0.), SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.))
        .set(active_camera(), 0.)
        .set(main_scene(), ())
        .spawn(world);

    EntityData::new()
        .set(translation(), vec3(-1., 0., 0.3))
        .set(rotation(), Quat::from_rotation_y(3.1) * Quat::from_rotation_z(PI))
        .set(light_ambient(), vec3(0.2, 0.2, 0.6) * 0.1)
        .set(light_diffuse(), vec3(250., 221., 152.) * 20. / 255.)
        .set(sun(), 0.)
        .set(main_scene(), ())
        .spawn(world);
}

fn main() {
    AppBuilder::simple().run(init);
}
