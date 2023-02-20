use ambient_asset_cache::AsyncAssetKeyExt;
use ambient_model_import::{fbx::FbxDoc, MaterialFilter, ModelImportPipeline, ModelImportTransform, ModelTransform};
use ambient_renderer::materials::pbr_material::PbrMaterialFromUrl;
use ambient_std::asset_url::{AbsAssetUrl, AssetType, AssetUrl};
use convert_case::{Case, Casing};
use futures::{future::BoxFuture, FutureExt};
use image::RgbaImage;
use itertools::Itertools;

use super::{
    super::{
        context::PipelineCtx,
        out_asset::{OutAssetContent, OutAssetPreview},
    },
    ModelsPipeline,
};
use crate::pipelines::{
    materials::PipeImage,
    out_asset::{asset_id_from_url, OutAsset},
};

pub async fn pipeline(ctx: &PipelineCtx, config: ModelsPipeline) -> Vec<OutAsset> {
    ctx.process_files(
        |file| {
            file.extension() == Some("json".to_string())
                && file.path().to_string().contains(&format!("_{}_", file.path().file_stem().unwrap()))
        },
        move |ctx, file| {
            let config = config.clone();
            async move {
                let mut res = Vec::new();
                let quixel_id = QuixelId::from_full(file.last_dir_name().unwrap()).unwrap();
                let quixel_json: serde_json::Value = file.download_json(ctx.assets()).await.unwrap();
                let in_root_url = file.join(".").unwrap();
                let tags = quixel_json["tags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| x.as_str().unwrap().to_string().to_case(Case::Title))
                    .collect_vec();
                let pack_name = quixel_json["semanticTags"]["name"].as_str().unwrap().to_string();
                let objs = object_pipelines_from_quixel_json(
                    &quixel_json,
                    quixel_id,
                    &ctx,
                    &config,
                    &in_root_url,
                    1.,
                    &ctx.out_root().join(ctx.in_root().relative_path(&file.path().join("0").join("material"))).unwrap().as_directory(),
                )
                .await
                .unwrap();
                let mut ids = Vec::new();
                let is_collection = objs.len() > 1;
                for (i, pipeline) in objs.into_iter().enumerate() {
                    let id = asset_id_from_url(&file.push(i.to_string()).unwrap());
                    let mut asset_crate = pipeline.produce_crate(ctx.assets()).await.unwrap();

                    let out_model_path = ctx.in_root().relative_path(file.path()).join(i.to_string());
                    config.apply(&ctx, &mut asset_crate, &out_model_path).await?;

                    let model_crate_url = ctx.write_model_crate(&asset_crate, &out_model_path).await;

                    res.push(OutAsset {
                        id: id.clone(),
                        type_: AssetType::Object,
                        hidden: is_collection,
                        name: pack_name.clone(),
                        tags: tags.clone(),
                        categories: Default::default(),

                        preview: OutAssetPreview::FromModel { url: model_crate_url.model().abs().unwrap() },
                        content: OutAssetContent::Content(model_crate_url.object().abs().unwrap()),
                        source: Some({
                            let mut f = file.clone();
                            f.0.set_fragment(Some(&i.to_string()));
                            f
                        }),
                    });
                    ids.push(id.to_string());
                }
                if is_collection {
                    res.push(OutAsset {
                        id: asset_id_from_url(&file),
                        type_: AssetType::Object,
                        hidden: false,
                        name: pack_name.to_string(),
                        tags,
                        categories: Default::default(),
                        preview: OutAssetPreview::None,
                        content: OutAssetContent::Collection(ids),
                        source: Some(file.clone()),
                    });
                }
                Ok(res)
            }
        },
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn object_pipelines_from_quixel_json(
    quixel: &serde_json::Value,
    quixel_id: QuixelId,
    ctx: &PipelineCtx,
    config: &ModelsPipeline,
    in_root_url: &AbsAssetUrl,
    lod_factor: f32,
    out_materials_url: &AbsAssetUrl,
) -> anyhow::Result<Vec<ModelImportPipeline>> {
    let mut object_defs = Vec::new();
    let pack_name = quixel["semanticTags"]["name"].as_str().unwrap().to_string();

    fn rougness_to_mr(img: &mut RgbaImage) {
        for p in img.pixels_mut() {
            p[0] = 255; // Metallic to 1 so that it's controlled by the pbr parameter instead
        }
    }

    let pipe_image = |ending: &str| -> BoxFuture<'_, anyhow::Result<AssetUrl>> {
        let ctx = ctx.clone();
        let in_root_url = in_root_url.clone();
        let config = config.clone();
        let ending = ending.to_string();
        async move {
            let pattern = format!("{}**/*{}", in_root_url.as_directory().path(), ending);
            let file = ctx.files.find_file_res(&pattern)?.clone();
            Ok(AssetUrl::from(PipeImage::new(file).cap_texture_size(config.cap_texture_sizes).get(ctx.assets()).await?))
        }
        .boxed()
    };
    match get_path(quixel, vec!["semanticTags", "asset_type"]).unwrap().as_str().unwrap() as &str {
        "3D asset" => {
            let material = PbrMaterialFromUrl {
                base_color: Some(pipe_image(&format!("{}_Albedo.jpg", quixel_id.resolution)).await?),
                opacity: if quixel_has_opacity(quixel).unwrap_or(false) {
                    Some(pipe_image(&format!("{}_Opacity.jpg", quixel_id.resolution)).await?)
                } else {
                    None
                },
                normalmap: Some(pipe_image(&format!("{}_Normal_LOD0.jpg", quixel_id.resolution)).await?),
                metallic_roughness: Some(pipe_image(&format!("{}_Roughness.jpg", quixel_id.resolution)).await?),
                roughness: 1.0,
                metallic: 0.2,
                ..Default::default()
            };
            let material_override = ModelImportTransform::OverrideMaterial {
                filter: MaterialFilter::All,
                material: Box::new(material.relative_path_from(out_materials_url)),
            };
            let mesh0 =
                FbxDoc::from_url(ctx.assets(), ctx.files.find_file_res(format!("{}**/*_LOD5.fbx", in_root_url.as_directory().path()))?)
                    .await?;
            for root_node in mesh0.models.values().filter(|m| m.parent.is_none()) {
                let mut lods = Vec::new();
                for i in 0..6 {
                    let root = root_node.node_name.clone().replace("LOD5", &format!("LOD{i}"));
                    lods.push(
                        ModelImportPipeline::new()
                            .add_step(ModelImportTransform::ImportModelFromUrl {
                                url: ctx.files.find_file_res(format!("{}**/*_LOD{i}.fbx", in_root_url.as_directory().path()))?.clone(),
                                normalize: true,
                                force_assimp: config.force_assimp,
                            })
                            .add_step(ModelImportTransform::Transform(ModelTransform::SetRoot { name: root })),
                    );
                }
                let object = ModelImportPipeline::new()
                    .add_step(ModelImportTransform::MergeMeshLods { lods, lod_cutoffs: None })
                    .add_step(material_override.clone())
                    .add_step(ModelImportTransform::CreateObject)
                    .add_step(ModelImportTransform::CreateColliderFromModel)
                    .add_step(ModelImportTransform::SetName { name: pack_name.clone() });
                object_defs.push(object);
            }
        }
        "3D plant" => {
            let get_map_v1 = |path: &str, name: &str| {
                if let Some(path) = get_path(quixel, vec![path]) {
                    path.as_array().unwrap().iter().find_map(|map| {
                        let map = map.as_object().unwrap();
                        if map.get("mimeType").unwrap().as_str().unwrap() == "image/jpeg"
                            && map.get("uri").unwrap().as_str().unwrap().contains(&format!("_{}_", quixel_id.resolution.to_uppercase()))
                            && map.get("name").unwrap().as_str().unwrap() == name
                        {
                            Some(map.get("uri").unwrap().as_str().unwrap().to_string())
                        } else {
                            None
                        }
                    })
                } else {
                    None
                }
            };

            let get_map_v2 = |path: &str, name: &str| {
                if let Some(path) = get_path(quixel, vec![path]) {
                    path.as_array().unwrap().iter().find_map(|map| {
                        let map = map.as_object().unwrap();
                        if map.get("name").unwrap().as_str().unwrap() == name {
                            for uri in map.get("uris").unwrap().as_array().unwrap() {
                                for resolution in uri.get("resolutions").unwrap().as_array().unwrap() {
                                    let resolution = resolution.as_object().unwrap();
                                    for format in resolution.get("formats").unwrap().as_array().unwrap() {
                                        let format = format.as_object().unwrap();
                                        tracing::info!("format={}", format.get("mimeType").unwrap().as_str().unwrap());
                                        tracing::info!("uri={}", format.get("uri").unwrap().as_str().unwrap());
                                        if format.get("mimeType").unwrap().as_str().unwrap() == "image/jpeg"
                                            && format
                                                .get("uri")
                                                .unwrap()
                                                .as_str()
                                                .unwrap()
                                                .contains(&format!("_{}_", quixel_id.resolution.to_uppercase()))
                                        {
                                            return Some(format.get("uri").unwrap().as_str().unwrap().to_string());
                                        }
                                    }
                                }
                            }
                        }
                        None
                    })
                } else {
                    None
                }
            };
            let pipe_image_opt = |ending: Option<String>| -> BoxFuture<'_, anyhow::Result<Option<AssetUrl>>> {
                async move {
                    if let Some(ending) = ending {
                        Ok(Some(pipe_image(&ending).await?))
                    } else {
                        Ok(None)
                    }
                }
                .boxed()
            };
            let pipe_mr_image = |ending: Option<String>| -> BoxFuture<'_, anyhow::Result<Option<AssetUrl>>> {
                let config = config.clone();
                let in_root_url = in_root_url.clone();
                let ending = ending;
                async move {
                    if let Some(ending) = ending {
                        Ok(Some(AssetUrl::from(
                            PipeImage::new(ctx.get_downloadable_url(&in_root_url.push(ending).unwrap()).unwrap().clone())
                                .transform("mr", |img, _| rougness_to_mr(img))
                                .cap_texture_size(config.cap_texture_sizes)
                                .get(ctx.assets())
                                .await?,
                        )))
                    } else {
                        Ok(None)
                    }
                }
                .boxed()
            };

            let atlas = PbrMaterialFromUrl {
                base_color: pipe_image_opt(get_map_v1("maps", "Albedo").or_else(|| get_map_v2("components", "Albedo"))).await?,
                opacity: pipe_image_opt(get_map_v1("maps", "Opacity").or_else(|| get_map_v2("components", "Opacity"))).await?,
                normalmap: pipe_image_opt(get_map_v1("maps", "Normal").or_else(|| get_map_v2("components", "Normal"))).await?,
                alpha_cutoff: Some(0.5),
                metallic_roughness: pipe_mr_image(get_map_v1("maps", "Roughness").or_else(|| get_map_v2("components", "Roughness")))
                    .await?,
                metallic: 0.2,
                roughness: 1.0,
                ..Default::default()
            }
            .relative_path_from(out_materials_url);
            let billboard = PbrMaterialFromUrl {
                base_color: pipe_image_opt(get_map_v1("billboards", "Albedo")).await?,
                opacity: pipe_image_opt(get_map_v1("billboards", "Opacity")).await?,
                normalmap: pipe_image_opt(get_map_v1("billboards", "Normal")).await?,
                alpha_cutoff: Some(0.5),
                metallic_roughness: pipe_mr_image(get_map_v1("maps", "Roughness")).await?,
                metallic: 0.2,
                roughness: 1.0,
                ..Default::default()
            }
            .relative_path_from(out_materials_url);
            for meta in get_path(quixel, vec!["meta"]).unwrap().as_array().unwrap() {
                if meta.as_object().unwrap().get("key").unwrap().as_str().unwrap() == "lodDistance" {
                    for variation in meta.as_object().unwrap().get("value").unwrap().as_array().unwrap() {
                        let variation = variation.as_object().unwrap();
                        let variation_nr = variation.get("variation").unwrap().as_u64().unwrap();
                        let json_lods = variation.get("distance").unwrap().as_array().unwrap();
                        let mut lods = Vec::new();
                        let mut lod_cutoffs = Vec::new();
                        for (i, lod) in json_lods.iter().enumerate() {
                            let is_last = i == json_lods.len() - 1;
                            let lod = lod.as_object().unwrap();
                            let lod_index = lod.get("lod").unwrap().as_u64().unwrap();
                            let lod_distance = lod.get("lodDistance").unwrap().as_f64().unwrap() as f32;
                            lod_cutoffs.push(lod_distance * lod_factor);
                            let file = in_root_url.push(format!("Var{variation_nr}/Var{variation_nr}_LOD{lod_index}.fbx")).unwrap();
                            lods.push(
                                ModelImportPipeline::new()
                                    .add_step(ModelImportTransform::ImportModelFromUrl {
                                        url: ctx.get_downloadable_url(&file).unwrap().clone(),
                                        normalize: true,
                                        force_assimp: config.force_assimp,
                                    })
                                    .add_step(ModelImportTransform::OverrideMaterial {
                                        filter: MaterialFilter::All,
                                        material: Box::new(if !is_last { atlas.clone() } else { billboard.clone() }),
                                    }),
                            );
                        }
                        let object = ModelImportPipeline::new()
                            .add_step(ModelImportTransform::MergeMeshLods { lods, lod_cutoffs: Some(lod_cutoffs) })
                            .add_step(ModelImportTransform::CreateObject)
                            .add_step(ModelImportTransform::CreateColliderFromModel)
                            .add_step(ModelImportTransform::SetName { name: pack_name.clone() });
                        object_defs.push(object);
                    }
                }
            }
            // Sometimes quixel.json doesn't define lodDistance in the meta
            if object_defs.is_empty() {
                let mut variation = 0;
                loop {
                    if !ctx
                        .files
                        .has_input_file(&in_root_url.push(format!("Var{var}/Var{var}_LOD{lod}.fbx", var = variation, lod = 0)).unwrap())
                    {
                        break;
                    }
                    let lods = (0..6)
                        .map(|i| {
                            ModelImportPipeline::new().add_step(ModelImportTransform::ImportModelFromUrl {
                                url: ctx
                                    .get_downloadable_url(&in_root_url.push(format!("Var{variation}/Var{variation}_LOD{i}.fbx")).unwrap())
                                    .unwrap()
                                    .clone(),
                                normalize: true,
                                force_assimp: config.force_assimp,
                            })
                        })
                        .collect();
                    let object = ModelImportPipeline::new()
                        .add_step(ModelImportTransform::MergeMeshLods { lods, lod_cutoffs: None })
                        .add_step(ModelImportTransform::OverrideMaterial { filter: MaterialFilter::All, material: Box::new(atlas.clone()) })
                        .add_step(ModelImportTransform::CreateObject)
                        .add_step(ModelImportTransform::CreateColliderFromModel)
                        .add_step(ModelImportTransform::SetName { name: pack_name.clone() });

                    object_defs.push(object);
                    variation += 1;
                }
            }
        }
        _ => panic!("Not implemented yet: {:?}", get_path(quixel, vec!["semanticTags", "asset_type"])),
    }
    Ok(object_defs)
}

#[derive(Clone, Debug)]
pub struct QuixelId {
    pub full: String,
    pub id: String,
    pub resolution: String,
    pub name: String,
}
impl QuixelId {
    /// Parses a quixel id from something like "Props_Storage_vijncb3_2K_3d_ms"
    pub fn from_full(full: &str) -> Option<Self> {
        let mut ss = full.split('_').collect_vec();
        ss.pop()?; // ms
        ss.pop()?; // 3dplant
        let resolution = ss.pop()?.to_string();
        let id = ss.pop()?.to_string();
        Some(Self { full: full.to_string(), resolution, id, name: ss.join(" ") })
    }
}

fn get_path<'a>(value: &'a serde_json::Value, mut path: Vec<&str>) -> Option<&'a serde_json::Value> {
    let p = path.remove(0);
    let o = value.as_object()?.get(p);
    if !path.is_empty() {
        get_path(o?, path)
    } else {
        o
    }
}
fn quixel_has_opacity(quixel: &serde_json::Value) -> Option<bool> {
    if let Some(components) = get_path(quixel, vec!["components"]) {
        for c in components.as_array()? {
            if c.as_object()?.get("type")?.as_str()? == "opacity" {
                return Some(true);
            }
        }
    }
    if let Some(components) = get_path(quixel, vec!["maps"]) {
        for c in components.as_array()? {
            if c.as_object()?.get("type")?.as_str()? == "opacity" {
                return Some(true);
            }
        }
    }
    if let Some(components) = get_path(quixel, vec!["billboards"]) {
        for c in components.as_array()? {
            if c.as_object()?.get("type")?.as_str()? == "opacity" {
                return Some(true);
            }
        }
    }
    Some(false)
}
