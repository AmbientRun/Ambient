use ambient_core::async_ecs::async_run;
use std::{str::FromStr, sync::Arc};

use ambient_core::{asset_cache, main_scene, mesh, runtime};
use ambient_ecs::{components, query, Entity, SystemGroup};
use ambient_gpu::{
    gpu::{Gpu, GpuKey},
    shader_module::{BindGroupDesc, ShaderModule},
    texture::Texture,
    texture_loaders::TextureFromUrl,
};
use ambient_meshes::QuadMeshKey;
use ambient_renderer::{
    color, material, renderer_shader, Material, MaterialShader, RendererConfig, RendererShader,
    SharedMaterial, StandardShaderKey, MATERIAL_BIND_GROUP,
};
use ambient_std::{
    asset_cache::{AssetCache, AsyncAssetKeyExt, SyncAssetKey, SyncAssetKeyExt},
    asset_url::AbsAssetUrl,
    cb, friendly_id,
};
use glam::Vec4;
use wgpu::BindGroup;

pub(crate) static OLD_CONTENT_SERVER_URL: &str =
    "https://fra1.digitaloceanspaces.com/dims-content/";

pub use ambient_ecs::generated::rendering::components::water;

components!("rendering", {
    water_normals: Arc<Texture>,
});

pub fn systems() -> SystemGroup {
    SystemGroup::new(
        "water",
        vec![
            query(water()).excl(renderer_shader()).to_system(|q, world, qs, _| {
                let runtime = world.resource(runtime()).clone();
                for (id, _) in q.collect_cloned(world, qs) {
                    let async_run = world.resource(async_run()).clone();
                    let assets = world.resource(asset_cache()).clone();
                    runtime.spawn(async move {
                        let normals = TextureFromUrl {
                            url: AbsAssetUrl::from_str(&format!(
                                "{OLD_CONTENT_SERVER_URL}assets/models/Cadhatch/seamless-water-textures/water 0342normal.jpg"
                            ))
                            .unwrap(),
                            format: wgpu::TextureFormat::Rgba8Unorm,
                        }
                        .get(&assets)
                        .await
                        .unwrap();
                        async_run.run(move |world| {
                            world.add_component(id, water_normals(), normals).unwrap();
                        })
                    });
                }
            }),
            query((water(), water_normals())).spawned().to_system(|q, world, qs, _| {
                let assets = world.resource(asset_cache()).clone();
                for (id, (_, normals)) in q.collect_cloned(world, qs) {
                    let data = Entity::new()
                        .with_merge(ambient_primitives::quad_data(&assets))
                        .with(renderer_shader(), cb(get_water_shader))
                        .with(material(), WaterMaterialKey::new(normals).get(&assets))
                        .with(main_scene(), ())
                        .with(mesh(), QuadMeshKey.get(&assets))
                        .with(color(), Vec4::ONE);
                    world.add_components(id, data).unwrap();
                }
            }),
        ],
    )
}

fn get_water_layout() -> BindGroupDesc<'static> {
    BindGroupDesc {
        entries: vec![wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        }],
        label: MATERIAL_BIND_GROUP.into(),
    }
}

#[derive(Debug)]
pub struct WaterMaterialShaderKey;
impl SyncAssetKey<Arc<MaterialShader>> for WaterMaterialShaderKey {
    fn load(&self, _assets: AssetCache) -> Arc<MaterialShader> {
        Arc::new(MaterialShader {
            id: "water_shader".to_string(),
            shader: Arc::new(
                ShaderModule::new(
                    "water_scattering",
                    [
                        ambient_sky::ATMOSPHERIC_SCATTERING_SOURCE,
                        include_str!("water.wgsl"),
                    ]
                    .concat(),
                )
                .with_binding_desc(get_water_layout()),
            ),
        })
    }
}

pub fn get_water_shader(assets: &AssetCache, config: &RendererConfig) -> Arc<RendererShader> {
    StandardShaderKey {
        material_shader: WaterMaterialShaderKey.get(assets),
        lit: true,
        shadow_cascades: config.shadow_cascades,
    }
    .get(assets)
}

#[derive(Debug)]
pub struct WaterMaterialKey {
    normals: Arc<Texture>,
}
impl WaterMaterialKey {
    pub fn new(normals: Arc<Texture>) -> Self {
        Self { normals }
    }
}
impl SyncAssetKey<SharedMaterial> for WaterMaterialKey {
    fn load(&self, assets: AssetCache) -> SharedMaterial {
        let gpu = GpuKey.get(&assets);
        SharedMaterial::new(WaterMaterial::new(&gpu, &assets, self.normals.clone()))
    }
}

#[derive(Debug)]
pub struct WaterMaterial {
    id: String,
    pub bind_group: wgpu::BindGroup,
}
impl WaterMaterial {
    pub fn new(gpu: &Gpu, assets: &AssetCache, normals: Arc<Texture>) -> Self {
        let layout = get_water_layout().get(assets);

        Self {
            id: friendly_id(),
            bind_group: gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &normals.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                }],
                label: Some("WaterMaterial.bind_group"),
            }),
        }
    }
}
impl Material for WaterMaterial {
    fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn transparent(&self) -> Option<bool> {
        Some(true)
    }
    fn double_sided(&self) -> Option<bool> {
        Some(true)
    }
}
