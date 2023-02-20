use ambient_app::AppBuilder;
use ambient_core::{asset_cache, camera::active_camera, main_scene, transform::*};
use ambient_ecs::World;
use ambient_element::ElementComponentExt;
use ambient_primitives::Quad;
use ambient_renderer::color;
use ambient_std::math::SphericalCoords;
use glam::*;

fn init(world: &mut World) {
    let _assets = world.resource(asset_cache()).clone();
    Quad.el().set(color(), vec4(0.5, 0.5, 0.5, 1.)).set(scale(), vec3(2., 2., 1.)).spawn_static(world);
    Quad.el()
        .set(color(), vec4(1., 0., 0., 1.))
        .set_default(spherical_billboard())
        .set(translation(), vec3(-1., 0., 1.))
        .set(scale(), vec3(0.5, 0.5, 0.5))
        .spawn_static(world);
    Quad.el()
        .set(color(), vec4(1., 0., 0., 1.))
        .set_default(cylindrical_billboard_z())
        .set(translation(), vec3(1., 0., 1.))
        .set(scale(), vec3(0.5, 0.5, 0.5))
        .spawn_static(world);

    ambient_cameras::spherical::new(vec3(0., 0., 0.), SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.))
        .set(active_camera(), 0.)
        .set(main_scene(), ())
        .spawn(world);
}

fn main() {
    AppBuilder::simple().run_world(init);
}
