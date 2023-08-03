use ambient_app::{App, AppBuilder};
use ambient_core::{camera::active_camera, main_scene};
use ambient_gizmos::{gizmos, GizmoPrimitive};
use ambient_native_std::math::SphericalCoords;
use ambient_std::line_uid;
use glam::*;

async fn init(app: &mut App) {
    let world = &mut app.world;

    world
        .resource(gizmos())
        .scope(line_uid!())
        .draw(GizmoPrimitive::sphere(vec3(0., 0., 0.), 1.));

    ambient_cameras::spherical::new(
        vec3(0., 0., 0.),
        SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.),
    )
    .with(active_camera(), 0.)
    .with(main_scene(), ())
    .spawn(world);
}

fn main() {
    AppBuilder::simple().block_on(init);
}
