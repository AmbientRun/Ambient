use ambient_app::{App, AppBuilder};
use ambient_core::{asset_cache, camera::active_camera, main_scene, transform::*};
use ambient_ecs::{query_mut, Entity};
use ambient_gpu::std_assets::{DefaultNormalMapViewKey, PixelTextureViewKey};
use ambient_meshes::{CubeMeshKey, SphereMeshKey};
use ambient_renderer::{
    color, gpu_primitives_lod, gpu_primitives_mesh,
    materials::pbr_material::{get_pbr_shader, PbrMaterial, PbrMaterialConfig, PbrMaterialParams},
    primitives, RenderPrimitive, SharedMaterial,
};
use ambient_std::{asset_cache::SyncAssetKeyExt, cb, color::Color, math::SphericalCoords};
use ambient_sys::time::Instant;
use glam::*;

async fn init(app: &mut App) {
    let world = &mut app.world;
    let assets = world.resource(asset_cache()).clone();
    let size = 5;

    for x in 0..size {
        for y in 0..size {
            let mat = SharedMaterial::new(PbrMaterial::new(
                assets.clone(),
                PbrMaterialConfig {
                    source: "".to_string(),
                    name: "".to_string(),
                    params: PbrMaterialParams::default(),
                    base_color: PixelTextureViewKey::white().get(&assets),
                    normalmap: DefaultNormalMapViewKey.get(&assets),
                    metallic_roughness: PixelTextureViewKey {
                        color: uvec4((255. * x as f32 / size as f32) as u32, (255. * y as f32 / size as f32) as u32, 0, 0),
                    }
                    .get(&assets),
                    transparent: None,
                    double_sided: None,
                    depth_write_enabled: None,
                },
            ));

            Entity::new()
                .with(
                    primitives(),
                    vec![RenderPrimitive { shader: cb(get_pbr_shader), material: mat.clone(), mesh: CubeMeshKey.get(&assets), lod: 0 }],
                )
                .with(color(), Vec4::ONE)
                .with_default(gpu_primitives_mesh())
                .with_default(gpu_primitives_lod())
                .with(main_scene(), ())
                .with_default(local_to_world())
                .with_default(mesh_to_world())
                .with(translation(), vec3(x as f32, y as f32, 0.))
                .with(scale(), Vec3::ONE * 0.4)
                .spawn(world);

            Entity::new()
                .with(
                    primitives(),
                    vec![RenderPrimitive {
                        shader: cb(get_pbr_shader),
                        material: mat,
                        mesh: SphereMeshKey::default().get(&assets),
                        lod: 0,
                    }],
                )
                .with(color(), Vec4::ONE)
                .with_default(gpu_primitives_mesh())
                .with_default(gpu_primitives_lod())
                .with(main_scene(), ())
                .with_default(local_to_world())
                .with_default(mesh_to_world())
                .with(translation(), vec3(x as f32, y as f32, 2.))
                .with(scale(), Vec3::ONE * 0.4)
                .spawn(world);
        }
    }

    ambient_cameras::spherical::new(vec3(0., 0., 0.), SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.))
        .with(active_camera(), 0.)
        .with(main_scene(), ())
        .spawn(world);

    let start = Instant::now();
    app.add_system(query_mut(color(), ()).to_system(move |q, w, qs, _| {
        let t = start.elapsed().as_secs_f32();

        let color = Color::hsl(t * 10.0 % 360.0, t.sin() * 0.5 + 0.5, 0.5).as_rgba_f32().into();
        for (_, c, ()) in q.iter(w, qs) {
            *c = color;
        }
    }));
}

fn main() {
    // wgpu_subscriber::initialize_default_subscriber(None);
    env_logger::init();
    AppBuilder::simple().block_on(init);
}
