use elements_app::{App, AppBuilder};
use elements_core::{
    asset_cache, camera::{active_camera, far}, main_scene, transform::*
};
use elements_ecs::World;
use elements_element::ElementComponentExt;
use elements_model_import::model_crate::ModelCrate;
use elements_primitives::Quad;
use elements_std::{asset_url::ContentUrl, math::SphericalCoords};
use glam::*;

async fn init(world: &mut World) {
    let assets = world.resource(asset_cache()).clone();

    Quad.el().set(scale(), Vec3::ONE * 30.).spawn_static(world);

    let model = ModelCrate::local_import(
        &assets,
        &ContentUrl::parse("https://dims-content.fra1.digitaloceanspaces.com/assets/models/MixamoCharacters/Vanguard By T. Choonyung.fbx")
            .unwrap(),
        true,
        true,
    )
    .await
    .unwrap();

    model.spawn(world, &Default::default());

    elements_cameras::spherical::new(vec3(0., 0., 0.), SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.))
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
