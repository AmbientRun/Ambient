use std::f32::consts::PI;

use elements_app::{App, AppBuilder};
use elements_core::{
    asset_cache, camera::active_camera, main_scene, transform::{rotation, scale, translation}
};
use elements_ecs::World;
use elements_element::ElementComponentExt;
use elements_primitives::{Cube, Quad};
use elements_renderer::{
    cast_shadows, color, material, materials::{
        flat_material::FlatMaterial, pbr_material::{PbrMaterial, PbrMaterialShaderKey}
    }, renderer_shader, DecalShaderKey, SharedMaterial
};
use elements_std::{asset_cache::SyncAssetKeyExt, math::SphericalCoords};
use glam::*;

fn init(world: &mut World) {
    Cube.el().set(color(), vec4(0.5, 0.5, 0.5, 1.)).set(translation(), Vec3::Z).set_default(cast_shadows()).spawn_static(world);
    Quad.el().set(scale(), Vec3::ONE * 10.).spawn_static(world);

    let assets = world.resource(asset_cache()).clone();

    Cube.el()
        .set(scale(), vec3(2., 2., 4.))
        .set(rotation(), Quat::from_rotation_y(PI / 4.) * Quat::from_rotation_z(PI / 4.))
        .init(renderer_shader(), DecalShaderKey { material_shader: PbrMaterialShaderKey.get(&assets), lit: true }.get(&assets))
        .init(material(), PbrMaterial::base_color_from_file(&assets, "assets/checkerboard.png").into())
        .spawn_static(world);

    let transparent = SharedMaterial::new(FlatMaterial::new(assets, vec4(0., 1., 0., 0.5), Some(true)));
    Cube.el()
        .set(scale(), vec3(2., 2., 4.))
        .set(rotation(), Quat::from_rotation_y(PI / 4.) * Quat::from_rotation_z(PI / 4.))
        .set(material(), transparent)
        .spawn_static(world);

    elements_cameras::spherical::new(vec3(0., 0., 0.), SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.))
        .set(active_camera(), 0.)
        .set(main_scene(), ())
        .spawn(world);
}

fn main() {
    env_logger::init();
    AppBuilder::simple().run_world(init);
}
