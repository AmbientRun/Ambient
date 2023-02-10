use glam::*;
use kiwi_app::AppBuilder;
use kiwi_core::{asset_cache, camera::active_camera, main_scene};
use kiwi_ecs::World;
use kiwi_element::ElementComponentExt;
use kiwi_meshes::{CubeMeshKey, SphereMeshKey};
use kiwi_primitives::Cube;
use kiwi_renderer::lod::{cpu_lod, lod_cutoffs, mesh_lods};
use kiwi_std::{asset_cache::SyncAssetKeyExt, math::SphericalCoords};

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

    kiwi_cameras::spherical::new(vec3(0., 0., 0.), SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.))
        .set(active_camera(), 0.)
        .set(main_scene(), ())
        .spawn(world);
}

fn main() {
    env_logger::init();
    AppBuilder::simple().run_world(init);
}
