use ambient_app::{App, AppBuilder};
use ambient_core::{
    asset_cache, bounding::{local_bounding_aabb, world_bounding_aabb, world_bounding_sphere}, camera::active_camera, main_scene, transform::{local_to_world, mesh_to_world, translation}
};
use ambient_ecs::Entity;
use ambient_meshes::{CubeMeshKey, SphereMeshKey};
use ambient_renderer::{
    color, flat_material::{get_flat_shader, FlatMaterialKey}, gpu_primitives, lod::{gpu_lod, lod_cutoffs}, primitives, RenderPrimitive
};
use ambient_std::{asset_cache::SyncAssetKeyExt, cb, math::SphericalCoords, shapes::AABB};
use glam::*;

async fn init(app: &mut App) {
    let world = &mut app.world;
    let assets = world.resource(asset_cache()).clone();
    let white_mat = FlatMaterialKey::white().get(&assets);
    let red_mat = FlatMaterialKey::new(vec4(1., 0., 0., 1.), Some(false)).get(&assets);

    let aabb = AABB { min: -Vec3::ONE * 0.5, max: Vec3::ONE * 0.5 };
    Entity::new()
        .with_default(local_to_world())
        .with_default(mesh_to_world())
        .with_default(translation())
        .with_default(gpu_primitives())
        .with(color(), Vec4::ONE)
        .with(main_scene(), ())
        .with(local_bounding_aabb(), aabb)
        .with(world_bounding_sphere(), aabb.to_sphere())
        .with(world_bounding_aabb(), aabb)
        .with(
            primitives(),
            vec![
                RenderPrimitive { mesh: CubeMeshKey.get(&assets), material: white_mat.clone(), shader: cb(get_flat_shader), lod: 0 },
                RenderPrimitive {
                    mesh: SphereMeshKey(Default::default()).get(&assets),
                    material: white_mat.clone(),
                    shader: cb(get_flat_shader),
                    lod: 1,
                },
                RenderPrimitive { mesh: CubeMeshKey.get(&assets), material: red_mat.clone(), shader: cb(get_flat_shader), lod: 2 },
                RenderPrimitive {
                    mesh: SphereMeshKey(Default::default()).get(&assets),
                    material: red_mat.clone(),
                    shader: cb(get_flat_shader),
                    lod: 3,
                },
            ],
        )
        .with(lod_cutoffs(), {
            let mut lods = vec![1., 0.5, 0.2, 0.];
            lods.resize(20, 0.);
            lods.try_into().unwrap()
        })
        .with_default(gpu_lod())
        .spawn(world);

    ambient_cameras::spherical::new(vec3(0., 0., 0.), SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.))
        .with(active_camera(), 0.)
        .with(main_scene(), ())
        .spawn(world);
}

fn main() {
    env_logger::init();
    AppBuilder::simple().block_on(init);
}
