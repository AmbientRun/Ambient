use glam::*;
use kiwi_app::AppBuilder;
use kiwi_core::{camera::active_camera, main_scene};
use kiwi_ecs::World;
use kiwi_gizmos::{gizmos, GizmoPrimitive};
use kiwi_std::{line_hash, math::SphericalCoords};

fn init(world: &mut World) {
    world.resource(gizmos()).scope(line_hash!()).draw(GizmoPrimitive::sphere(vec3(0., 0., 0.), 1.));

    kiwi_cameras::spherical::new(vec3(0., 0., 0.), SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.))
        .set(active_camera(), 0.)
        .set(main_scene(), ())
        .spawn(world);
}

fn main() {
    AppBuilder::simple().run_world(init);
}
