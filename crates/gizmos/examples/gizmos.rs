use elements_app::AppBuilder;
use elements_core::{camera::active_camera, main_scene};
use elements_ecs::World;
use elements_gizmos::{gizmos, GizmoPrimitive};
use elements_std::{line_hash, math::SphericalCoords};
use glam::*;

fn init(world: &mut World) {
    world.resource(gizmos()).scope(line_hash!()).draw(GizmoPrimitive::sphere(vec3(0., 0., 0.), 1.));

    elements_cameras::spherical::new(vec3(0., 0., 0.), SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.))
        .set(active_camera(), 0.)
        .set(main_scene(), ())
        .spawn(world);
}

fn main() {
    AppBuilder::simple().run_world(init);
}
