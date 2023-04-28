use ambient_app::{AmbientWindow, AppBuilder};
use ambient_core::{asset_cache, camera::active_camera, main_scene, transform::*};
use ambient_element::ElementComponentExt;
use ambient_primitives::{Cube, Quad};
use ambient_renderer::{material, materials::flat_material::FlatMaterial, SharedMaterial};
use ambient_std::math::SphericalCoords;
use glam::*;

async fn init(app: &mut AmbientWindow) {
    let world = &mut app.world;
    let assets = world.resource(asset_cache()).clone();

    let red = FlatMaterial::new(assets.clone(), vec4(1., 0., 0., 1.), Some(false));
    let grey = FlatMaterial::new(assets.clone(), vec4(0.5, 0.5, 0.5, 1.), Some(false));
    let transparent =
        SharedMaterial::new(FlatMaterial::new(assets, vec4(0., 1., 0., 0.5), Some(true)));

    Cube.el()
        .with(material(), SharedMaterial::new(grey))
        .spawn_static(world);
    Quad.el()
        .with(material(), SharedMaterial::new(red))
        .with(scale(), vec3(2., 2., 1.))
        .spawn_static(world);
    Cube.el()
        .with(material(), transparent.clone())
        .with(translation(), vec3(0., 0., 2.))
        .with(scale(), vec3(0.2, 2., 1.))
        .spawn_static(world);
    Cube.el()
        .with(material(), transparent)
        .with(translation(), vec3(4., 0., 0.))
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
    env_logger::init();
    AppBuilder::simple().block_on(init);
}
