use elements_app::{App, AppBuilder};
use elements_core::{asset_cache, camera::active_camera, main_scene, transform::*};
use elements_ecs::World;
use elements_element::ElementComponentExt;
use elements_primitives::{Cube, Quad};
use elements_renderer::{material, materials::flat_material::FlatMaterial, SharedMaterial};
use elements_std::math::SphericalCoords;
use glam::*;

fn init(world: &mut World) {
    let assets = world.resource(asset_cache()).clone();

    let red = FlatMaterial::new(assets.clone(), vec4(1., 0., 0., 1.), Some(false));
    let grey = FlatMaterial::new(assets.clone(), vec4(0.5, 0.5, 0.5, 1.), Some(false));
    let transparent = SharedMaterial::new(FlatMaterial::new(assets, vec4(0., 1., 0., 0.5), Some(true)));

    Cube.el().set(material(), SharedMaterial::new(grey)).spawn_static(world);
    Quad.el().set(material(), SharedMaterial::new(red)).set(scale(), vec3(2., 2., 1.)).spawn_static(world);
    Cube.el().set(material(), transparent.clone()).set(translation(), vec3(0., 0., 2.)).set(scale(), vec3(0.2, 2., 1.)).spawn_static(world);
    Cube.el().set(material(), transparent).set(translation(), vec3(4., 0., 0.)).spawn_static(world);

    elements_cameras::spherical::new(vec3(0., 0., 0.), SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.))
        .set(active_camera(), 0.)
        .set(main_scene(), ())
        .spawn(world);
}

fn main() {
    env_logger::init();
    AppBuilder::simple().run_world(init);
}
