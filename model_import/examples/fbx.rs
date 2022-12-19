use elements::{self, app::App, cameras, ecs::World, model::bones_to_lines, primitives::Quad};
use elements_core::{
    asset_cache, camera::{active_camera, far}, hierarchy::dump_world_hierarchy_to_tmp_file, main_scene, transform::*
};
use elements_element::ElementComponentExt;
use elements_gizmos::{gizmos, GizmoPrimitive};
use elements_model_import::{model_crate::ModelCrate, ModelImportPipeline, ModelImportTransform};
use elements_std::{line_hash, math::SphericalCoords};
use glam::*;

async fn init(world: &mut World) {
    let assets = world.resource(asset_cache()).clone();

    Quad.el().set(scale(), Vec3::ONE * 30.).spawn_static(world);

    let model = ModelCrate::local_import(
        &assets,
        "/Users/fredrik/My project/Assets/NatureManufacture Assets/Dynamic Nature - Mountain Tree Pack/Models/Fir_01_Plant.FBX",
        false,
        false,
    )
    .await
    .unwrap()
    .produce_local_model(&assets)
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

    cameras::spherical::new(vec3(0., 0., 0.), SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.))
        .set(active_camera(), 0.)
        .set(main_scene(), ())
        .set(far(), 2000.)
        .spawn(world);
}

fn main() {
    env_logger::init();
    App::run_debug_app_with_config(false, true, true, |app, runtime| {
        runtime.block_on(async { init(&mut app.world).await });
    });
}
