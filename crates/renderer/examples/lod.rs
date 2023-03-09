//! Provides a sphere with varying LOD
//!
//! # Expected behavior
//!
//! When zooming out, the sphere should gradually decrease in subdivisions, and the hue should change gradually from 0 degrees to 360 degrees

use ambient_app::{App, AppBuilder};
use ambient_core::{
    asset_cache,
    bounding::{local_bounding_aabb, world_bounding_aabb, world_bounding_sphere},
    camera::active_camera,
    main_scene,
    transform::{local_to_world, mesh_to_world, translation},
};
use ambient_ecs::Entity;
use ambient_meshes::{SphereMeshKey, UVSphereMesh};
use ambient_renderer::{
    color,
    flat_material::{get_flat_shader, FlatMaterialKey},
    gpu_primitives,
    lod::{gpu_lod, lod_cutoffs, LodCutoffs},
    primitives, RenderPrimitive,
};
use ambient_std::{asset_cache::SyncAssetKeyExt, cb, color::Color, math::SphericalCoords, shapes::AABB};
use glam::*;

async fn init(app: &mut App) {
    let world = &mut app.world;
    let assets = world.resource(asset_cache()).clone();
    const LODS: usize = 16;

    let default_min_screen_size = 0.04; // i.e. 4%
    let lod_step = (1f32 / default_min_screen_size).powf(1. / (LODS - 1) as f32);

    let (prims, lods): (Vec<_>, Vec<_>) = (0..LODS)
        .map(|i| {
            let detail = LODS - i;
            let mesh = SphereMeshKey(UVSphereMesh { radius: 1.0, sectors: 3 * detail + 2, stacks: detail + 2 }).get(&assets);
            let f = i as f32 / LODS as f32;

            let material = FlatMaterialKey::new(Color::hsl(f * 360.0, 1.0, 0.5).as_linear_rgba_f32().into(), Some(false)).get(&assets);

            (RenderPrimitive { mesh, material, shader: cb(get_flat_shader), lod: i }, 1. / lod_step.powi(i as i32))
        })
        .unzip();

    log::info!("Lod levels: {lods:?}");

    let aabb = AABB { min: -Vec3::ONE, max: Vec3::ONE };
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
        .with(primitives(), prims)
        .with(lod_cutoffs(), LodCutoffs::new(&lods))
        .with_default(gpu_lod())
        .spawn(world);

    ambient_cameras::spherical::new(vec3(0., 0., 0.), SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.))
        .with(active_camera(), 0.)
        .with(main_scene(), ())
        .spawn(world);
}

fn main() {
    tracing_subscriber::fmt::init();
    AppBuilder::simple().block_on(init);
}
