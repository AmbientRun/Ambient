use ambient_app::{App, AppBuilder};
use ambient_core::{
    asset_cache,
    camera::{active_camera, far},
    main_scene,
    transform::*,
};
use ambient_element::ElementComponentExt;
use ambient_model_import::model_crate::ModelCrate;
use ambient_primitives::Quad;
use ambient_std::{asset_url::AbsAssetUrl, math::SphericalCoords};
use glam::*;

async fn init(app: &mut App) {
    let world = &mut app.world;
    let assets = world.resource(asset_cache()).clone();

    Quad.el().set(scale(), Vec3::ONE * 30.).spawn_static(world);

    let model = ModelCrate::local_import(
        &assets,
        &AbsAssetUrl::parse("https://dims-content.fra1.digitaloceanspaces.com/assets/models/MixamoCharacters/Vanguard By T. Choonyung.fbx")
            .unwrap(),
        true,
        true,
    )
    .await
    .unwrap();

    model.spawn(world, &Default::default());

    ambient_cameras::spherical::new(vec3(0., 0., 0.), SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.))
        .set(active_camera(), 0.)
        .set(main_scene(), ())
        .set(far(), 2000.)
        .spawn(world);
}

fn main() {
    env_logger::init();
    AppBuilder::simple().block_on(init);
}
