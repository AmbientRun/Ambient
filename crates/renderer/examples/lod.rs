use elements_app::AppBuilder;
use elements_core::{asset_cache, camera::active_camera, main_scene};
use elements_ecs::World;
use elements_element::ElementComponentExt;
use elements_meshes::{CubeMeshKey, SphereMeshKey};
use elements_primitives::Cube;
use elements_renderer::lod::{cpu_lod, lod_cutoffs, mesh_lods};
use elements_std::{asset_cache::SyncAssetKeyExt, math::SphericalCoords};
use glam::*;

fn init(world: &mut World) {
    let assets = world.resource(asset_cache()).clone();

    Cube.el()
        .set(
            mesh_lods(),
            vec![
                CubeMeshKey.get(&assets),
                SphereMeshKey(Default::default()).get(&assets),
                CubeMeshKey.get(&assets),
                SphereMeshKey(Default::default()).get(&assets),
            ],
        )
        .set(lod_cutoffs(), {
            let mut lods = vec![1., 0.5, 0.2, 0.];
            lods.resize(20, 0.);
            lods.try_into().unwrap()
        })
        .set(cpu_lod(), 0_usize)
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
