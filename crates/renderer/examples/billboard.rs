use ambient_app::{App, AppBuilder};
use ambient_core::{asset_cache, camera::active_camera, main_scene, transform::*};
use ambient_element::ElementComponentExt;
use ambient_native_std::math::SphericalCoords;
use ambient_primitives::Quad;
use ambient_renderer::color;
use glam::*;

async fn init(app: &mut App) {
    let world = &mut app.world;
    let _assets = world.resource(asset_cache()).clone();
    Quad.el()
        .with(color(), vec4(0.5, 0.5, 0.5, 1.))
        .with(scale(), vec3(2., 2., 1.))
        .spawn_static(world);
    Quad.el()
        .with(color(), vec4(1., 0., 0., 1.))
        .with(spherical_billboard(), ())
        .with(translation(), vec3(-1., 0., 1.))
        .with(scale(), vec3(0.5, 0.5, 0.5))
        .spawn_static(world);
    Quad.el()
        .with(color(), vec4(1., 0., 0., 1.))
        .with(cylindrical_billboard_z(), ())
        .with(translation(), vec3(1., 0., 1.))
        .with(scale(), vec3(0.5, 0.5, 0.5))
        .spawn_static(world);

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
