use ambient_app::{App, AppBuilder};
use ambient_core::{asset_cache, camera::active_camera, main_scene};
use ambient_element::ElementComponentExt;
use ambient_meshes::{CubeMeshKey, SphereMeshKey};
use ambient_primitives::Cube;
use ambient_renderer::lod::{cpu_lod, gpu_lod, lod_cutoffs, mesh_lods};
use ambient_std::{asset_cache::SyncAssetKeyExt, math::SphericalCoords};
use glam::*;

async fn init(app: &mut App) {
    let world = &mut app.world;
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
            lods.resize(16, 0.);
            lods.try_into().unwrap()
        })
        .set(cpu_lod(), 0_usize)
        .set(gpu_lod(), ())
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
