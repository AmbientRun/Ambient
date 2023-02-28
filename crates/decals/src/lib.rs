use std::sync::Arc;

use ambient_asset_cache::{AssetCache, AsyncAssetKeyExt, SyncAssetKey, SyncAssetKeyExt};
use ambient_core::{
    asset_cache,
    async_ecs::async_run,
    bounding::{local_bounding_aabb, world_bounding_aabb, world_bounding_sphere},
    main_scene, mesh, runtime,
    transform::{local_to_world, mesh_to_world},
};
use ambient_ecs::{components, query, Entity, MakeDefault, Networked, Store, SystemGroup};
use ambient_gpu::shader_module::{Shader, ShaderModule};
use ambient_meshes::CubeMeshKey;
use ambient_renderer::{
    color, get_forward_module, gpu_primitives, material,
    pbr_material::{PbrMaterialFromUrl, PbrMaterialShaderKey},
    primitives, renderer_shader, MaterialShader, RendererShader,
};
use ambient_std::{
    asset_url::{MaterialAssetType, TypedAssetUrl},
    cb,
    download_asset::JsonFromUrl,
    include_file,
    shapes::AABB,
    unwrap_log_err, unwrap_log_warn,
};
use ambient_ui::Editable;
use glam::{Vec3, Vec4};

components!("decals", {
    @[MakeDefault, Editable, Networked, Store]
    decal: TypedAssetUrl<MaterialAssetType>,
});

pub struct DecalShaderKey {
    pub material_shader: Arc<MaterialShader>,
    pub lit: bool,
    pub shadow_cascades: u32,
}
impl std::fmt::Debug for DecalShaderKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DecalShaderKey").field("material_shader", &self.material_shader.id).field("lit", &self.lit).finish()
    }
}
impl SyncAssetKey<Arc<RendererShader>> for DecalShaderKey {
    fn load(&self, assets: AssetCache) -> Arc<RendererShader> {
        let id = format!("decal_shader_{}_{}", self.material_shader.id, self.lit);
        let shader = Shader::from_modules(
            &assets,
            id.clone(),
            [
                &get_forward_module(&assets, self.shadow_cascades),
                &self.material_shader.shader,
                &ShaderModule::new("DecalMaterial", include_file!("decal.wgsl"), vec![]),
            ]
            .into_iter(),
        );

        Arc::new(RendererShader {
            shader,
            id,
            vs_main: "vs_main".to_string(),
            fs_shadow_main: "fs_shadow_main".to_string(),
            fs_forward_main: if self.lit { "fs_forward_lit_main".to_string() } else { "fs_forward_unlit_main".to_string() },
            fs_outline_main: "fs_outlines_main".to_string(),
            transparent: true,
            double_sided: true,
            depth_write_enabled: false,
            transparency_group: -100,
        })
    }
}

pub fn client_systems() -> SystemGroup {
    SystemGroup::new(
        "decals_client",
        vec![query(decal().changed()).to_system(|q, world, qs, _| {
            for (id, decal) in q.collect_cloned(world, qs) {
                let decal = if let Some(url) = decal.abs() {
                    url
                } else {
                    log::error!("Decal was not an absolute url: {}", decal);
                    continue;
                };
                let assets = world.resource(asset_cache()).clone();
                let async_run = world.resource(async_run()).clone();
                world.resource(runtime()).spawn(async move {
                    let mat_def = unwrap_log_warn!(JsonFromUrl::<PbrMaterialFromUrl>::new(decal.clone(), true).get(&assets).await);
                    let mat = unwrap_log_warn!(unwrap_log_err!(mat_def.resolve(&decal)).get(&assets).await);
                    async_run.run(move |world| {
                        let aabb = AABB { min: -Vec3::ONE, max: Vec3::ONE };
                        let mut data = Entity::new()
                            .with(material(), mat.into())
                            .with(
                                renderer_shader(),
                                cb(move |assets, config| {
                                    DecalShaderKey {
                                        material_shader: PbrMaterialShaderKey.get(assets),
                                        lit: true,
                                        shadow_cascades: config.shadow_cascades,
                                    }
                                    .get(assets)
                                }),
                            )
                            .with(mesh(), CubeMeshKey.get(&assets))
                            .with(primitives(), vec![])
                            .with_default(gpu_primitives())
                            .with(main_scene(), ())
                            .with(local_bounding_aabb(), aabb)
                            .with(world_bounding_sphere(), aabb.to_sphere())
                            .with(world_bounding_aabb(), aabb);
                        if !world.has_component(id, local_to_world()) {
                            data.set(local_to_world(), Default::default());
                        }
                        if !world.has_component(id, mesh_to_world()) {
                            data.set(mesh_to_world(), Default::default());
                        }
                        if !world.has_component(id, color()) {
                            data.set(color(), Vec4::ONE);
                        }
                        world.add_components(id, data).ok();
                    })
                });
            }
        })],
    )
}
