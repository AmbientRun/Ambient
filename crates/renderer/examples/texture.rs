use std::sync::Arc;

use elements_app::{gpu, AppBuilder};
use elements_core::{asset_cache, camera::active_camera, main_scene, transform::*};
use elements_ecs::{EntityData, World};
use elements_gpu::{
    std_assets::{DefaultNormalMapViewKey, PixelTextureViewKey}, texture::Texture
};
use elements_meshes::CubeMeshKey;
use elements_renderer::{
    gpu_primitives, materials::pbr_material::{get_pbr_shader, PbrMaterial, PbrMaterialConfig, PbrMaterialParams}, primitives, RenderPrimitive, SharedMaterial
};
use elements_std::{asset_cache::SyncAssetKeyExt, math::SphericalCoords};
use glam::*;

fn init(world: &mut World) {
    let gpu = world.resource(gpu()).clone();
    let assets = world.resource(asset_cache()).clone();

    let texture = Arc::new(
        Arc::new(Texture::from_file(gpu, "assets/checkerboard.png", wgpu::TextureFormat::Rgba8UnormSrgb))
            .create_view(&wgpu::TextureViewDescriptor::default()),
    );
    let mat = SharedMaterial::new(PbrMaterial::new(
        assets.clone(),
        PbrMaterialConfig {
            source: "".to_string(),
            name: "".to_string(),
            params: PbrMaterialParams::default(),
            base_color: texture,
            normalmap: DefaultNormalMapViewKey.get(&assets),
            metallic_roughness: PixelTextureViewKey::white().get(&assets),
            transparent: None,
            double_sided: None,
            depth_write_enabled: None,
        },
    ));

    EntityData::new()
        .set(primitives(), vec![RenderPrimitive { shader: get_pbr_shader(&assets), material: mat, mesh: CubeMeshKey.get(&assets), lod: 0 }])
        .set_default(gpu_primitives())
        .set(main_scene(), ())
        .set_default(local_to_world())
        .set_default(mesh_to_world())
        .spawn(world);

    elements_cameras::spherical::new(vec3(0., 0., 0.), SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.))
        .set(active_camera(), 0.)
        .set(main_scene(), ())
        .spawn(world);
}

fn main() {
    env_logger::init();
    AppBuilder::simple().run_world(init);
}
