use glam::*;
use kiwi_app::AppBuilder;
use kiwi_core::{
    asset_cache, camera::{active_camera, far}, main_scene, transform::*
};
use kiwi_ecs::World;
use kiwi_element::ElementComponentExt;
use kiwi_model_import::model_crate::ModelCrate;
use kiwi_primitives::Quad;
use kiwi_std::{asset_url::AbsAssetUrl, math::SphericalCoords};

async fn init(world: &mut World) {
    let assets = world.resource(asset_cache()).clone();

    Quad.el().set(scale(), Vec3::ONE * 30.).spawn_static(world);

    let model = ModelCrate::local_import(
        &assets,
        &AbsAssetUrl::parse("https://dims-content.fra1.digitaloceanspaces.com/assets/models/MixamoCharacters/Vanguard.glb").unwrap(),
        true,
        false,
    )
    .await
    .unwrap();

    model.spawn(world, &Default::default());

    kiwi_cameras::spherical::new(vec3(0., 0., 0.), SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.))
        .set(active_camera(), 0.)
        .set(main_scene(), ())
        .set(far(), 2000.)
        .spawn(world);
}

fn main() {
    env_logger::init();
    AppBuilder::simple().run(|app, runtime| {
        runtime.block_on(async { init(&mut app.world).await });
    });
}
