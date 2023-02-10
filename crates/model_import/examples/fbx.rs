use glam::*;
use kiwi_app::AppBuilder;
use kiwi_core::{
    asset_cache, camera::{active_camera, far}, hierarchy::dump_world_hierarchy_to_tmp_file, main_scene, transform::*
};
use kiwi_ecs::World;
use kiwi_element::ElementComponentExt;
use kiwi_gizmos::{gizmos, GizmoPrimitive};
use kiwi_model::bones_to_lines;
use kiwi_model_import::model_crate::ModelCrate;
use kiwi_primitives::Quad;
use kiwi_std::{asset_url::AbsAssetUrl, line_hash, math::SphericalCoords};

async fn init(world: &mut World) {
    let assets = world.resource(asset_cache()).clone();

    Quad.el().set(scale(), Vec3::ONE * 30.).spawn_static(world);

    let model = ModelCrate::local_import(
        &assets,
        &AbsAssetUrl::parse("https://dims-content.fra1.digitaloceanspaces.com/assets/models/MixamoCharacters/Vanguard By T. Choonyung.fbx")
            .unwrap(),
        true,
        false,
    )
    .await
    .unwrap();
    dump_world_hierarchy_to_tmp_file(&model.0);

    let id = model.spawn(world, &Default::default());
    {
        let mut scope = world.resource(gizmos()).scope(line_hash!());
        for line in bones_to_lines(world, id) {
            scope.draw(GizmoPrimitive::line(line.0, line.1, 0.01));
        }
    }

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
