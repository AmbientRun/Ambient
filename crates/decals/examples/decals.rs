use std::f32::consts::PI;

use ambient_app::{App, AppBuilder};
use ambient_core::{
    asset_cache,
    camera::active_camera,
    main_scene,
    transform::{rotation, scale, translation},
};
use ambient_decals::DecalShaderKey;
use ambient_element::ElementComponentExt;
use ambient_primitives::{Cube, Quad};
use ambient_renderer::{
    cast_shadows, color, material,
    materials::{
        flat_material::FlatMaterial,
        pbr_material::{PbrMaterial, PbrMaterialShaderKey},
    },
    renderer_shader, SharedMaterial,
};
use ambient_std::{asset_cache::SyncAssetKeyExt, cb, math::SphericalCoords};
use glam::*;

async fn init(app: &mut App) {
    let world = &mut app.world;
    Cube.el().with(color(), vec4(0.5, 0.5, 0.5, 1.)).with(translation(), Vec3::Z).with_default(cast_shadows()).spawn_static(world);
    Quad.el().with(scale(), Vec3::ONE * 10.).spawn_static(world);

    let assets = world.resource(asset_cache()).clone();

    Cube.el()
        .with(scale(), vec3(2., 2., 4.))
        .with(rotation(), Quat::from_rotation_y(PI / 4.) * Quat::from_rotation_z(PI / 4.))
        .init(
            renderer_shader(),
            cb(move |assets, config| {
                DecalShaderKey { material_shader: PbrMaterialShaderKey.get(assets), lit: true, shadow_cascades: config.shadow_cascades }
                    .get(assets)
            }),
        )
        .init(material(), PbrMaterial::base_color_from_file(&assets, "assets/checkerboard.png").into())
        .spawn_static(world);

    let transparent = SharedMaterial::new(FlatMaterial::new(assets, vec4(0., 1., 0., 0.5), Some(true)));
    Cube.el()
        .with(scale(), vec3(2., 2., 4.))
        .with(rotation(), Quat::from_rotation_y(PI / 4.) * Quat::from_rotation_z(PI / 4.))
        .with(material(), transparent)
        .spawn_static(world);

    ambient_cameras::spherical::new(vec3(0., 0., 0.), SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.))
        .with(active_camera(), 0.)
        .with(main_scene(), ())
        .spawn(world);
}

fn main() {
    env_logger::init();
    AppBuilder::simple().block_on(init);
}
