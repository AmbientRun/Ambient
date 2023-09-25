use std::f32::consts::TAU;

use ambient_app::{App, AppBuilder};
use ambient_core::{asset_cache, camera::active_camera, gpu, main_scene, transform::*};
use ambient_ecs::{query_mut, Entity};
use ambient_gpu::{
    sampler::SamplerKey,
    std_assets::{DefaultNormalMapViewKey, PixelTextureViewKey},
};
use ambient_meshes::{CubeMeshKey, SphereMeshKey};
use ambient_native_std::{asset_cache::SyncAssetKeyExt, cb, color::Color, math::SphericalCoords};
use ambient_renderer::{
    color,
    flat_material::{get_flat_shader, FlatMaterialKey, FlatShaderKey},
    gpu_primitives_lod, gpu_primitives_mesh, light_ambient, light_diffuse, material,
    materials::pbr_material::{get_pbr_shader, PbrMaterial, PbrMaterialConfig, PbrMaterialParams},
    primitives, renderer_shader, sun, RenderPrimitive, SharedMaterial,
};
use ambient_sys::time::Instant;
use glam::*;
use itertools::Itertools;

async fn init(app: &mut App) {
    let world = &mut app.world;
    let gpu = world.resource(gpu()).clone();
    let assets = world.resource(asset_cache()).clone();
    let size = 9;
    let max = size - 1;

    // let red = SharedMaterial::new(FlatMaterial::new(
    //     &gpu,
    //     &assets,
    //     vec4(1., 0., 0., 1.),
    //     Some(false),
    // ));

    let metallic = SharedMaterial::new(PbrMaterial::new(
        &gpu,
        &assets,
        PbrMaterialConfig {
            source: "".to_string(),
            name: "".to_string(),
            params: PbrMaterialParams::default(),
            base_color: PixelTextureViewKey::white().get(&assets),
            normalmap: DefaultNormalMapViewKey.get(&assets),
            metallic_roughness: PixelTextureViewKey {
                color: uvec4(255, 150, 0, 0),
            }
            .get(&assets),
            sampler: SamplerKey::LINEAR_CLAMP_TO_EDGE.get(&assets),
            transparent: None,
            double_sided: None,
            depth_write_enabled: None,
        },
    ));

    for (((i, j), k), m) in (0..size)
        .cartesian_product(0..size)
        .cartesian_product(0..size)
        .cartesian_product([false, true])
    {
        let theta = (i + j * size) as f32 / (size * size) as f32 * TAU;
        let r = k as f32 + 2.0 + m as i32 as f32 * 0.5;

        let primitive = if m {
            vec![RenderPrimitive {
                shader: cb(get_pbr_shader),
                material: metallic.clone(),
                mesh: SphereMeshKey::default().get(&assets),
                lod: 0,
            }]
        } else {
            vec![RenderPrimitive {
                shader: cb(get_flat_shader),
                material: FlatMaterialKey::white().get(&assets),
                mesh: SphereMeshKey::default().get(&assets),
                lod: 0,
            }]
        };

        Entity::new()
            .with(primitives(), primitive)
            .with(material(), FlatMaterialKey::white().get(&assets))
            .with(renderer_shader(), cb(get_flat_shader))
            .with(
                color(),
                Color::hsl(
                    i as f32 / max as f32 * 360.0,
                    j as f32 / max as f32,
                    k as f32 / max as f32,
                )
                .as_rgba_f32()
                .into(),
            )
            .with(gpu_primitives_mesh(), Default::default())
            .with(gpu_primitives_lod(), Default::default())
            .with(main_scene(), ())
            .with(local_to_world(), Default::default())
            .with(mesh_to_world(), Default::default())
            .with(translation(), vec3(theta.sin() * r, theta.cos() * r, 0.0))
            .with(scale(), Vec3::ONE * 0.4)
            .spawn(world);
    }

    for x in 0..size {
        for y in 0..size {
            let mat = SharedMaterial::new(PbrMaterial::new(
                &gpu,
                &assets,
                PbrMaterialConfig {
                    source: "".to_string(),
                    name: "".to_string(),
                    params: PbrMaterialParams::default(),
                    base_color: PixelTextureViewKey::white().get(&assets),
                    normalmap: DefaultNormalMapViewKey.get(&assets),
                    metallic_roughness: PixelTextureViewKey {
                        color: uvec4(
                            (255. * x as f32 / (size - 1) as f32) as u32,
                            (255. * y as f32 / (size - 1) as f32) as u32,
                            0,
                            0,
                        ),
                    }
                    .get(&assets),
                    sampler: SamplerKey::LINEAR_CLAMP_TO_EDGE.get(&assets),
                    transparent: None,
                    double_sided: None,
                    depth_write_enabled: None,
                },
            ));

            Entity::new()
                .with(
                    primitives(),
                    vec![RenderPrimitive {
                        shader: cb(get_pbr_shader),
                        material: mat.clone(),
                        mesh: CubeMeshKey.get(&assets),
                        lod: 0,
                    }],
                )
                .with(color(), Vec4::ONE)
                .with(gpu_primitives_mesh(), Default::default())
                .with(gpu_primitives_lod(), Default::default())
                .with(main_scene(), ())
                .with(local_to_world(), Default::default())
                .with(mesh_to_world(), Default::default())
                .with(translation(), vec3(x as f32, y as f32 + 16.0, 0.))
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
                .with(gpu_primitives_mesh(), Default::default())
                .with(gpu_primitives_lod(), Default::default())
                .with(main_scene(), ())
                .with(local_to_world(), Default::default())
                .with(mesh_to_world(), Default::default())
                .with(translation(), vec3(x as f32, y as f32 + 16.0, 2.))
                .with(scale(), Vec3::ONE * 0.4)
                .spawn(world);
        }
    }

    ambient_cameras::spherical::new(
        vec3(0., 0., 0.),
        SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.),
    )
    .with(active_camera(), 0.)
    .with(main_scene(), ())
    .spawn(world);

    Entity::new()
        .with(sun(), 0.0)
        .with(rotation(), Quat::from_rotation_y(-1.))
        .with(main_scene(), ())
        .with(light_diffuse(), Vec3::ONE * 2.0)
        .with(light_ambient(), Vec3::ZERO)
        .spawn(world);

    // let start = Instant::now();

    // app.add_system(query_mut(color(), ()).to_system(move |q, w, qs, _| {
    //     let t = start.elapsed().as_secs_f32();

    //     let color = Color::hsl(t * 10.0 % 360.0, 0.5, 0.5).as_rgba_f32().into();

    //     for (_, c, ()) in q.iter(w, qs) {
    //         *c = color;
    //     }
    // }));
}

fn main() {
    // wgpu_subscriber::initialize_default_subscriber(None);
    env_logger::init();
    AppBuilder::simple().block_on(init);
}
