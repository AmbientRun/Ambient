use std::{collections::HashMap, io::Cursor, sync::Arc};

use anyhow::Context;
use async_recursion::async_recursion;
use elements_core::{
    hierarchy::{children, parent}, name, transform::{get_world_transform, mesh_to_local, rotation, scale, translation}
};
use elements_ecs::{EntityData, EntityId, World};
use elements_model::{pbr_renderer_primitives_from_url, Model, PbrRenderPrimitiveFromUrl};
use elements_model_import::{
    dotdot_path, model_crate::{cap_texture_size, ModelCrate}, ModelImportPipeline, ModelImportTransform, ModelTransform, RelativePathBufExt
};
use elements_renderer::{
    lod::{gpu_lod, lod_cutoffs}, materials::pbr_material::PbrMaterialFromUrl
};
use elements_std::{
    asset_cache::AssetCache, asset_url::{AbsAssetUrl, AssetType, ModelCrateAssetType, TypedAssetUrl}
};
use futures::{future::join_all, FutureExt};
use glam::{Mat4, Vec3, Vec4};
use image::ImageOutputFormat;
use itertools::Itertools;
use relative_path::RelativePathBuf;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use unity_parser::{parse_unity_yaml, prefab::PrefabObject, UnityRef};
use yaml_rust::Yaml;

use super::{super::context::PipelineCtx, create_texture_resolver, ModelsPipeline};
use crate::pipelines::{download_image, OutAsset, OutAssetContent, OutAssetPreview};

#[derive(Debug, Serialize, Deserialize)]
pub struct UnityConfig {
    #[serde(default)]
    use_prefabs: bool,
}

pub async fn pipeline(ctx: &PipelineCtx, use_prefabs: bool, config: ModelsPipeline) -> Vec<OutAsset> {
    let guid_lookup = join_all(
        ctx.process_ctx
            .files
            .iter()
            .cloned()
            .filter(|file| file.path().starts_with(ctx.in_root().path()))
            .filter_map(|file| {
                if let Some(base_path) = file.path().as_str().strip_suffix(".meta") {
                    let base_path = base_path.to_string();
                    let mut base_file = file.clone();
                    base_file.set_path(base_path);
                    if let Ok(base_file) = ctx.get_downloadable_url(&base_file).cloned() {
                        Some(async move {
                            let docs = download_unity_yaml(&ctx.assets(), &file).await?;
                            Ok((docs[0]["guid"].as_str().unwrap().to_string(), base_file))
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect_vec(),
    )
    .await
    .into_iter()
    .collect::<anyhow::Result<HashMap<_, _>>>()
    .unwrap();

    log::info!("guid_lookup done");

    let materials = Arc::new(Mutex::new(UnityMaterials { materials: Default::default(), ctx: ctx.clone() }));
    let guid_lookup = Arc::new(guid_lookup);
    let mesh_models = Arc::new(Mutex::new(MeshModels { models: Default::default(), force_assimp: config.force_assimp }));

    if use_prefabs {
        ctx.process_files(
            |file| file.extension() == Some("prefab".to_string()),
            move |ctx, file| {
                let config = config.clone();
                let materials = materials.clone();
                let mesh_models = mesh_models.clone();
                let guid_lookup = guid_lookup.clone();
                async move {
                    let mut res = Vec::new();
                    let prefab =
                        unity_parser::prefab::PrefabFile::from_yaml(download_unity_yaml(&ctx.assets(), &file).await.unwrap()).unwrap();

                    let out_model_path = ctx.in_root().relative_path(file.path());
                    let out_model_url = ctx.out_root().join(&out_model_path).unwrap();

                    let mut asset_crate = model_from_prefab(
                        UnityCtx {
                            ctx: &ctx,
                            config: &config,
                            materials_lookup: &materials,
                            mesh_models: &mesh_models,
                            guid_lookup: &guid_lookup,
                        },
                        &prefab,
                        &out_model_url.into(),
                    )
                    .await?;
                    config.apply(&ctx, &mut asset_crate).await?;

                    let model_url = ctx.write_model_crate(&asset_crate, &out_model_path).await;
                    res.push(OutAsset {
                        id: file.to_string(),
                        type_: AssetType::Object,
                        hidden: false,
                        name: file.path().file_name().unwrap().to_string(),
                        tags: Default::default(),
                        categories: Default::default(),
                        preview: OutAssetPreview::FromModel { url: model_url.abs().unwrap() },
                        content: OutAssetContent::Content(model_url.model_crate().unwrap().object().abs().unwrap()),
                        source: Some(file.clone()),
                    });
                    Ok(res)
                }
            },
        )
        .await
    } else {
        // TODO(fred): Should parse .meta file to find ModelImporter instead of checking extension
        ctx.process_files(
            |file| file.extension() == Some("fbx".to_string()),
            move |ctx, file| {
                let config = config.clone();
                let materials = materials.clone();
                let guid_lookup = guid_lookup.clone();
                async move {
                    let mut res = Vec::new();

                    let out_path = ctx.in_root().relative_path(file.path());
                    let out_root = ctx.out_root().join(&out_path).unwrap();

                    let pipeline = ModelImportPipeline::new()
                        .add_step(ModelImportTransform::ImportModelFromUrl {
                            url: file.clone(),
                            normalize: true,
                            force_assimp: config.force_assimp,
                        })
                        .add_step(ModelImportTransform::SetName { name: file.path().file_name().unwrap().to_string() })
                        .add_step(ModelImportTransform::Transform(ModelTransform::Center))
                        .add_step(ModelImportTransform::CreateObject)
                        .add_step(ModelImportTransform::CreateColliderFromModel);
                    let mut asset_crate = pipeline.produce_crate(&ctx.assets()).await.unwrap();
                    for mat in asset_crate.materials.content.values_mut() {
                        let name = mat.name.clone().unwrap();
                        let material_url = file.join(format!("Materials/{}.mat", name)).unwrap();
                        *mat = materials.lock().await.get_unity_material(&config, &guid_lookup, &material_url, &name).await.unwrap();
                        *mat = mat.relative_path_from(&out_root.join(format!("materials/{}.json", name)).unwrap());
                    }

                    config.apply(&ctx, &mut asset_crate).await?;

                    let model_url = ctx.write_model_crate(&asset_crate, &out_path).await;
                    res.push(OutAsset {
                        id: file.to_string(),
                        type_: AssetType::Object,
                        hidden: false,
                        name: file.path().file_name().unwrap().to_string(),
                        tags: Default::default(),
                        categories: Default::default(),
                        preview: OutAssetPreview::FromModel { url: model_url.abs().unwrap().into() },
                        content: OutAssetContent::Content(model_url.model_crate().unwrap().object().abs().unwrap()),
                        source: Some(file.clone()),
                    });
                    Ok(res)
                }
            },
        )
        .await
    }
}

async fn download_unity_yaml(assets: &AssetCache, url: &AbsAssetUrl) -> anyhow::Result<Vec<Yaml>> {
    let data = url.download_string(assets).await?;
    parse_unity_yaml(&data)
}

#[derive(Clone, Copy)]
struct UnityCtx<'a> {
    ctx: &'a PipelineCtx,
    guid_lookup: &'a HashMap<String, AbsAssetUrl>,
    config: &'a ModelsPipeline,
    materials_lookup: &'a Mutex<UnityMaterials>,
    mesh_models: &'a Mutex<MeshModels>,
}

// Materials might be shared, so we need to process them globaly
// TODO(fred): Materials aren't standardized in Unity, so for instance
// metalic_r_ao_g_smothness_a might be called something else in another materials.
// Need to do something about that at some point
struct UnityMaterials {
    materials: HashMap<String, PbrMaterialFromUrl>,
    ctx: PipelineCtx,
}
impl UnityMaterials {
    async fn get_by_guid(
        &mut self,
        config: &ModelsPipeline,
        guid_lookup: &HashMap<String, AbsAssetUrl>,
        unity_ref: &UnityRef,
    ) -> anyhow::Result<PbrMaterialFromUrl> {
        let material_path = guid_lookup.get(unity_ref.guid.as_ref().unwrap()).unwrap();
        if unity_ref.type_ == Some(2) {
            self.get_unity_material(config, guid_lookup, material_path, unity_ref.guid.as_ref().unwrap())
                .await
                .with_context(|| format!("Failed to get material {material_path}"))
        } else if unity_ref.type_ == Some(3) {
            Ok(Default::default())
        } else {
            todo!()
        }
    }
    async fn get_unity_material(
        &mut self,
        config: &ModelsPipeline,
        guid_lookup: &HashMap<String, AbsAssetUrl>,
        material_url: &AbsAssetUrl,
        name: &str,
    ) -> anyhow::Result<PbrMaterialFromUrl> {
        if let Some(mat) = self.materials.get(&material_url.to_string()) {
            Ok(mat.clone())
        } else {
            let get_texture = |ref_: &Option<UnityRef>| -> Option<&AbsAssetUrl> {
                if let Some(UnityRef { guid: Some(guid), .. }) = ref_ {
                    if let Some(url) = guid_lookup.get(guid) {
                        return Some(url);
                    }
                }
                None
            };

            let docs = download_unity_yaml(self.ctx.assets(), &material_url).await?;
            let mat = unity_parser::mat::Material::from_yaml(&docs[0])?;
            let metallic_r_ao_g_smothness_a = if let Some(file) = get_texture(&mat.metallic_r_ao_g_smothness_a) {
                Some((download_image(self.ctx.assets(), &file).await?.into_rgba8(), file))
            } else {
                None
            };
            let metallic_gloss_map = if let Some(file) = get_texture(&mat.metallic_gloss_map) {
                Some((download_image(self.ctx.assets(), &file).await?.into_rgba8(), file))
            } else {
                None
            };
            let occlusion_map = if let Some(file) = get_texture(&mat.occlusion_map) {
                Some((download_image(self.ctx.assets(), &file).await?.into_rgba8(), file))
            } else {
                None
            };
            let base_color = if let Some(file) = get_texture(&mat.main_tex) {
                let mut image = download_image(self.ctx.assets(), &file).await?.into_rgba8();
                // Pre-multiply AO
                if let Some((maos, _)) = &occlusion_map {
                    for (b, m) in image.pixels_mut().zip(maos.pixels()) {
                        b[0] = ((m[0] as f32 / 255.) * b[0] as f32) as u8;
                        b[1] = ((m[0] as f32 / 255.) * b[1] as f32) as u8;
                        b[2] = ((m[0] as f32 / 255.) * b[2] as f32) as u8;
                    }
                } else if let Some((maos, _)) = &metallic_r_ao_g_smothness_a {
                    for (b, m) in image.pixels_mut().zip(maos.pixels()) {
                        b[0] = ((m[1] as f32 / 255.) * b[0] as f32) as u8;
                        b[1] = ((m[1] as f32 / 255.) * b[1] as f32) as u8;
                        b[2] = ((m[1] as f32 / 255.) * b[2] as f32) as u8;
                    }
                }
                Some((image, file.clone()))
            } else {
                None
            };
            let metallic_roughness = if let Some((mut maos, file)) = metallic_r_ao_g_smothness_a {
                // metalic_r_ao_g_smothness_a -> metalic_r_roughness_g
                for p in maos.pixels_mut() {
                    p[1] = p[3];
                }
                Some((maos, file.clone()))
            } else if let Some((mut mg, file)) = metallic_gloss_map {
                for p in mg.pixels_mut() {
                    p[1] = 255 - p[3];
                }
                Some((mg, file.clone()))
            } else {
                None
            };
            let normalmap = if let Some(file) = get_texture(&mat.bump_map) {
                let image = download_image(self.ctx.assets(), &file).await?.into_rgba8();
                Some((image, file.clone()))
            } else {
                None
            };
            let get_image = |image_and_file: Option<(image::RgbaImage, AbsAssetUrl)>| {
                if let Some((mut image, file)) = image_and_file {
                    let out_image_path = self.ctx.in_root().relative_path(&file.path()).prejoin("materials").with_extension("png");
                    let ctx = self.ctx.clone();
                    let config = config.clone();
                    async move {
                        let mut data = Cursor::new(Vec::new());
                        tokio::task::block_in_place(|| {
                            if let Some(size) = config.cap_texture_sizes {
                                cap_texture_size(&mut image, size.size());
                            }
                            image.write_to(&mut data, ImageOutputFormat::Png).unwrap();
                        });
                        Some(ctx.write_file(&out_image_path, data.into_inner()).await)
                    }
                    .boxed()
                } else {
                    async move { None as Option<AbsAssetUrl> }.boxed()
                }
            };
            let (base_color, normalmap, metallic_roughness) =
                futures::join!(get_image(base_color), get_image(normalmap), get_image(metallic_roughness));
            let mat = PbrMaterialFromUrl {
                name: Some(name.to_string()),
                source: None,
                base_color: base_color.map(|x| x.into()),
                normalmap: normalmap.map(|x| x.into()),
                metallic_roughness: metallic_roughness.map(|x| x.into()),
                opacity: None,
                base_color_factor: None,
                emissive_factor: None,
                transparent: None,
                alpha_cutoff: mat.alpha_cutoff,
                double_sided: Some(true), // TODO: Double sided is configured in the shader in unity, so hard to know. Maybe make user configureable
                metallic: 1.,
                roughness: 1.,
            };
            self.materials.insert(name.to_string(), mat.clone());
            Ok(mat)
        }
    }
}

async fn model_from_prefab(
    ctx: UnityCtx<'_>,
    prefab_file: &unity_parser::prefab::PrefabFile,
    out_model_url: &TypedAssetUrl<ModelCrateAssetType>,
) -> anyhow::Result<ModelCrate> {
    // std::fs::write("tmp/unity.yml", prefab_file.dump());
    let root_game_objects = prefab_file.get_root_game_objects();
    let model_crate = parking_lot::Mutex::new(ModelCrate::new());
    model_crate.lock().models.insert(ModelCrate::MAIN, Model(World::new("model")));
    let roots =
        join_all(root_game_objects.into_iter().map(|root_game_object| {
            recursively_create_game_objects(ctx, prefab_file, root_game_object, None, &model_crate, out_model_url)
        }))
        .await
        .into_iter()
        .collect::<anyhow::Result<Vec<_>>>()?;
    let mut model_crate = model_crate.into_inner();
    model_crate.model_world_mut().add_resource(children(), roots);
    model_crate.model_mut().transform(Mat4::from_cols(Vec4::Y, Vec4::Z, Vec4::X, Vec4::W));

    model_crate.create_object();
    model_crate.create_collider_from_model(&ctx.ctx.assets())?;
    model_crate.finalize_model();
    Ok(model_crate)
}

#[async_recursion]
async fn recursively_create_game_objects<'a: 'async_recursion>(
    ctx: UnityCtx<'a>,
    prefab: &unity_parser::prefab::PrefabFile,
    object: &unity_parser::prefab::GameObject,
    parent_id: Option<EntityId>,
    model_crate: &parking_lot::Mutex<ModelCrate>,
    out_model_url: &TypedAssetUrl<ModelCrateAssetType>,
) -> anyhow::Result<EntityId> {
    let go_transform =
        object.get_component::<unity_parser::prefab::Transform>(prefab).map(|t| t.absolute_transform(prefab)).unwrap_or_default();
    let mut has_lod_group = false;
    let mut node = if let Some(lod_group) = object.get_component::<unity_parser::prefab::LODGroup>(prefab) {
        has_lod_group = true;
        let mut model_lods = Vec::<PbrRenderPrimitiveFromUrl>::new();
        let mut cutoffs = Vec::new();
        for (lod_i, lod) in lod_group.lods.iter().enumerate() {
            let mesh_renderer = lod.get_renderer(prefab).unwrap();

            model_lods.extend(
                primitives_from_unity_mesh_renderer(ctx, prefab, mesh_renderer, model_crate, go_transform, lod_i, out_model_url)
                    .await?
                    .into_iter(),
            );
            cutoffs.push(lod.screen_relative_height);
        }
        cutoffs.resize(20, 0.);
        let cutoffs: [f32; 20] = cutoffs.try_into().unwrap();

        EntityData::new().set(lod_cutoffs(), cutoffs).set_default(gpu_lod()).set(pbr_renderer_primitives_from_url(), model_lods)
    } else if let Some(mesh_renderer) = object.get_component::<unity_parser::prefab::MeshRenderer>(prefab) {
        let primitives =
            primitives_from_unity_mesh_renderer(ctx, prefab, mesh_renderer, model_crate, go_transform, 0, out_model_url).await?;
        EntityData::new().set(lod_cutoffs(), [0.; 20]).set_default(gpu_lod()).set(pbr_renderer_primitives_from_url(), primitives)
    } else {
        EntityData::new()
    };

    node.set_self(name(), object.name.clone());

    if let Some(transform) = object.get_component::<unity_parser::prefab::Transform>(prefab) {
        node.set_self(scale(), transform.local_scale);
        node.set_self(rotation(), transform.local_rotation);
        node.set_self(translation(), transform.local_position);
    }
    if let Some(parent_id) = parent_id {
        node.set_self(parent(), parent_id);
    }
    let id = node.spawn(model_crate.lock().model_world_mut());
    if !has_lod_group {
        if let Some(transform) = object.get_component::<unity_parser::prefab::Transform>(prefab) {
            let childs = join_all(transform.children.iter().map(|c| async move {
                if let Some(PrefabObject::Transform(trans)) = prefab.objects.get(&c.file_id) {
                    if let Some(PrefabObject::GameObject(obj)) = prefab.objects.get(&trans.game_object.file_id) {
                        return Ok(Some(recursively_create_game_objects(ctx, prefab, obj, Some(id), model_crate, out_model_url).await?));
                    }
                }
                Ok(None)
            }))
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
            if !childs.is_empty() {
                model_crate.lock().model_world_mut().add_component(id, children(), childs).unwrap();
            }
        }
    }
    Ok(id)
}

struct MeshModels {
    models: HashMap<AbsAssetUrl, Arc<ModelCrate>>,
    force_assimp: bool,
}
impl MeshModels {
    async fn get(&mut self, ctx: &PipelineCtx, mesh_url: &AbsAssetUrl) -> anyhow::Result<Arc<ModelCrate>> {
        if !self.models.contains_key(mesh_url) {
            let mut tmp_model = ModelCrate::new();
            tmp_model.import(&ctx.assets(), mesh_url, false, self.force_assimp, create_texture_resolver(ctx)).await?;
            tmp_model.update_transforms();
            // dump_world_hierarchy_to_tmp_file(tmp_model.model_world());
            self.models.insert(mesh_url.clone(), Arc::new(tmp_model));
        }

        Ok(self.models.get(mesh_url).unwrap().clone())
    }
}

async fn primitives_from_unity_mesh_renderer(
    ctx: UnityCtx<'_>,
    prefab: &unity_parser::prefab::PrefabFile,
    mesh_renderer: &unity_parser::prefab::MeshRenderer,
    model_crate: &parking_lot::Mutex<ModelCrate>,
    go_transform: Mat4,
    lod_i: usize,
    out_model_url: &TypedAssetUrl<ModelCrateAssetType>,
) -> anyhow::Result<Vec<PbrRenderPrimitiveFromUrl>> {
    let game_object = mesh_renderer.get_game_object(prefab).unwrap();
    let mesh_filter = game_object.get_component::<unity_parser::prefab::MeshFilter>(prefab).unwrap();
    let mesh_guid = mesh_filter.mesh.guid.as_ref().unwrap().clone();
    let mesh_url = ctx.guid_lookup.get(&mesh_guid).unwrap().clone();
    let mesh_meta_url = ctx.ctx.get_downloadable_url(&mesh_url.add_extension("meta")).unwrap();
    let mesh_meta = download_unity_yaml(&ctx.ctx.assets(), &mesh_meta_url).await.unwrap();
    if mesh_url.extension_is("asset") {
        let asset = download_unity_yaml(&ctx.ctx.assets(), &mesh_url).await.unwrap();
        let mut mesh = unity_parser::asset::Asset::from_yaml(asset[0].clone()).mesh;
        let mat_ref = mesh_renderer.materials[0].clone();
        let mut mat = ctx.materials_lookup.lock().await.get_by_guid(ctx.config, ctx.guid_lookup, &mat_ref).await.unwrap();
        mat = mat.relative_path_from(&out_model_url.material("x").abs().unwrap());
        mesh.transform(Mat4::from_cols(-Vec4::X, Vec4::Z, -Vec4::Y, Vec4::W));
        let mut model_crate = model_crate.lock();
        Ok(vec![PbrRenderPrimitiveFromUrl {
            mesh: dotdot_path(model_crate.meshes.insert(mesh_guid.clone(), mesh).path).into(),
            material: Some(dotdot_path(model_crate.materials.insert(mat_ref.to_string(), mat).path).into()),
            lod: lod_i,
        }])
    } else {
        let model_importer = unity_parser::model_importer::ModelImporter::from_yaml(&mesh_meta[0]).unwrap();
        let root_node = model_importer.id_to_name.get(&mesh_filter.mesh.file_id).unwrap();
        let tmp_model = ctx.mesh_models.lock().await.get(ctx.ctx, &mesh_url).await?;
        let node = tmp_model.model().get_entity_id_by_name(root_node).unwrap();
        let prims = tmp_model.model_world().get_ref(node, pbr_renderer_primitives_from_url()).unwrap();
        let mut res = Vec::new();
        for (prim, mat_ref) in prims.iter().zip(mesh_renderer.materials.iter()) {
            let mut mat = ctx.materials_lookup.lock().await.get_by_guid(ctx.config, ctx.guid_lookup, mat_ref).await.unwrap();
            mat = mat.relative_path_from(&out_model_url.material("x").abs().unwrap());
            let mut mesh = tmp_model.meshes.get_by_path(prim.mesh.path()).unwrap().clone();
            let node_world = get_world_transform(tmp_model.model_world(), node).unwrap();
            let node_mesh_transform = tmp_model.model_world().get(node, mesh_to_local()).unwrap_or_default();
            let transform = tmp_model.model().get_transform().unwrap_or_default() * node_world * node_mesh_transform;
            let file_scale = if model_importer.use_file_scale { Mat4::from_scale(Vec3::ONE * 0.01) } else { Mat4::IDENTITY };
            // This is a complete guess, but basically we're "zeroing" the game objects absolute transform, and "only" using the
            // models absolute transform
            // Also, TODO(fred): We should probably get rid of mesh_to_local and put the mesh transform on the primitive,
            // so that we don't need to transform the actual mesh here, but instead just put the transform on the primitive
            // below
            mesh.transform(go_transform.inverse() * file_scale * transform);
            mesh.invert_indicies();
            let mut model_crate = model_crate.lock();
            res.push(PbrRenderPrimitiveFromUrl {
                mesh: dotdot_path(model_crate.meshes.insert(format!("{}_{}", mesh_guid, prim.mesh.path()), mesh).path).into(),
                material: Some(dotdot_path(model_crate.materials.insert(mat_ref.to_string(), mat).path).into()),
                lod: lod_i,
            });
        }
        Ok(res)
    }
}
