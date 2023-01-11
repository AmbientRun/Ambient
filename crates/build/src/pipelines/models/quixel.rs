use convert_case::{Case, Casing};
use elements_model_import::{
    fbx::FbxDoc, model_crate::ModelCrate, MaterialFilter, ModelImportPipeline, ModelImportTransform, ModelTransform, RelativePathBufExt
};
use elements_renderer::materials::pbr_material::PbrMaterialFromUrl;
use elements_std::asset_url::{AssetType, AssetUrl};
use image::RgbaImage;
use itertools::Itertools;
use relative_path::RelativePathBuf;

use super::{
    super::{
        context::PipelineCtx, out_asset::{OutAssetContent, OutAssetPreview}
    }, ModelsPipeline
};
use crate::{
    helpers::download_json, pipelines::{materials::pipe_image, OutAsset}
};

pub async fn pipeline(ctx: &PipelineCtx, config: ModelsPipeline) -> Vec<OutAsset> {
    ctx.process_files(
        |file| {
            file.sub_path.extension == Some("json".to_string()) && file.sub_path_string.contains(&format!("_{}_", file.sub_path.filename))
        },
        move |ctx, file| {
            let config = config.clone();
            async move {
                let mut res = Vec::new();
                let quixel_id = QuixelId::from_full(file.sub_path.path.last().unwrap()).unwrap();
                let quixel_json: serde_json::Value = download_json(&ctx.assets, &file.temp_download_url).await.unwrap();
                let base_path = file.sub_path.path.join("/");
                let tags = quixel_json["tags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| x.as_str().unwrap().to_string().to_case(Case::Title))
                    .collect_vec();
                let pack_name = quixel_json["semanticTags"]["name"].as_str().unwrap().to_string();
                let materials_id = ctx.asset_crate_id(&format!("{}_material", file.sub_path_string));
                let _materials_url = ctx.crate_url(&materials_id);
                let mut materials = ModelCrate::new();
                let objs = object_pipelines_from_quixel_json(
                    &quixel_json,
                    quixel_id,
                    &ctx,
                    &config,
                    &base_path,
                    1.,
                    &mut materials,
                    &materials_id.crate_uid,
                )
                .await
                .unwrap();
                let mut ids = Vec::new();
                let is_collection = objs.len() > 1;
                for (i, pipeline) in objs.into_iter().enumerate() {
                    let asset_crate_id = ctx.asset_crate_id(&format!("{}{}", file.sub_path_string, i));
                    let asset_crate_url = ctx.crate_url(&asset_crate_id);
                    let mut asset_crate = pipeline.produce_crate(&ctx.assets).await.unwrap();

                    config.apply(&ctx, &mut asset_crate).await?;

                    res.push(OutAsset {
                        asset_crate_id: asset_crate_id.clone(),
                        sub_asset: None,
                        type_: AssetType::Object,
                        hidden: is_collection,
                        name: pack_name.clone(),
                        tags: tags.clone(),
                        categories: Default::default(),
                        preview: OutAssetPreview::FromModel {
                            url: asset_crate_url.resolve(asset_crate.models.loc.path(ModelCrate::MAIN).as_str()).unwrap(),
                        },
                        content: OutAssetContent::Content(
                            asset_crate_url.resolve(asset_crate.objects.loc.path(ModelCrate::MAIN).as_str()).unwrap(),
                        ),
                        source: Some(format!("{}#{}", file.sub_path_string, i)),
                    });
                    ctx.write_model_crate(&asset_crate, &asset_crate_id).await;
                    ids.push(asset_crate_id);
                }
                if is_collection {
                    res.push(OutAsset {
                        asset_crate_id: ctx.asset_crate_id(&file.sub_path_string),
                        sub_asset: None,
                        type_: AssetType::Object,
                        hidden: false,
                        name: pack_name.to_string(),
                        tags,
                        categories: Default::default(),
                        preview: OutAssetPreview::None,
                        content: OutAssetContent::Collection(ids),
                        source: Some(file.sub_path_string.clone()),
                    });
                }
                if let Some(max_size) = config.cap_texture_sizes {
                    materials.cap_texture_sizes(max_size.size());
                }
                ctx.write_model_crate(&materials, &materials_id).await;
                Ok(res)
            }
        },
    )
    .await
}

fn material_image_path(materials_id: &str, path: RelativePathBuf) -> AssetUrl {
    path.prejoin(format!("../../{materials_id}")).into()
}

#[allow(clippy::too_many_arguments)]
pub async fn object_pipelines_from_quixel_json(
    quixel: &serde_json::Value,
    quixel_id: QuixelId,
    ctx: &PipelineCtx,
    config: &ModelsPipeline,
    base_path: &str,
    lod_factor: f32,
    materials: &mut ModelCrate,
    materials_id: &str,
) -> anyhow::Result<Vec<ModelImportPipeline>> {
    let mut object_defs = Vec::new();
    let pack_name = quixel["semanticTags"]["name"].as_str().unwrap().to_string();

    fn rougness_to_mr(img: &mut RgbaImage) {
        for p in img.pixels_mut() {
            p[0] = 255; // Metallic to 1 so that it's controlled by the pbr parameter instead
        }
    }

    match get_path(quixel, vec!["semanticTags", "asset_type"]).unwrap().as_str().unwrap() as &str {
        "3D asset" => {
            let find_mesh_base_name = || {
                for mesh in quixel["meshes"].as_array().unwrap() {
                    if mesh["type"].as_str().unwrap() == "lod" {
                        for uri in mesh["uris"].as_array().unwrap() {
                            let uri_str = uri["uri"].as_str().unwrap();
                            if uri_str.to_lowercase().ends_with(".fbx") {
                                return uri_str[0..uri_str.len() - 9].to_string();
                            }
                        }
                    }
                }
                quixel_id.id.to_string()
            };

            let mut mesh_base_name = find_mesh_base_name();
            if !ctx.files.contains_key(&format!("{base_path}/{mesh_base_name}_LOD0.fbx")) {
                mesh_base_name = quixel_id.id.clone();
            }
            log::info!("Loading 3d asset: {:?}", mesh_base_name);
            let material_override = ModelImportTransform::OverrideMaterial {
                filter: MaterialFilter::All,
                material: Box::new(PbrMaterialFromUrl {
                    base_color: pipe_image(
                        ctx,
                        materials,
                        Some(format!("{base_path}/{}_{}_Albedo.jpg", mesh_base_name, quixel_id.resolution)),
                        |_| {},
                    )
                    .await?
                    .map(|x| material_image_path(materials_id, x)),
                    opacity: if quixel_has_opacity(quixel).unwrap_or(false) {
                        pipe_image(
                            ctx,
                            materials,
                            Some(format!("{base_path}/{}_{}_Opacity.jpg", mesh_base_name, quixel_id.resolution)),
                            |_| {},
                        )
                        .await?
                        .map(|x| material_image_path(materials_id, x))
                    } else {
                        None
                    },
                    normalmap: pipe_image(
                        ctx,
                        materials,
                        Some(format!("{base_path}/{}_{}_Normal_LOD0.jpg", mesh_base_name, quixel_id.resolution)),
                        |_| {},
                    )
                    .await?
                    .map(|x| material_image_path(materials_id, x)),
                    metallic_roughness: pipe_image(
                        ctx,
                        materials,
                        Some(format!("{base_path}/{}_{}_Roughness.jpg", mesh_base_name, quixel_id.resolution)),
                        rougness_to_mr,
                    )
                    .await?
                    .map(|x| material_image_path(materials_id, x)),
                    roughness: 1.0,
                    metallic: 0.2,
                    ..Default::default()
                }),
            };
            let mesh0 =
                FbxDoc::from_url(&ctx.assets, &ctx.files.get(&format!("{base_path}/{mesh_base_name}_LOD5.fbx")).unwrap().temp_download_url)
                    .await?;
            for root_node in mesh0.models.values().filter(|m| m.parent.is_none()) {
                let mut lods = Vec::new();
                for i in 0..6 {
                    let root = root_node.node_name.clone().replace("LOD5", &format!("LOD{i}"));
                    lods.push(
                        ModelImportPipeline::new()
                            .add_step(ModelImportTransform::ImportModelFromUrl {
                                url: ctx.files.get(&format!("{base_path}/{mesh_base_name}_LOD{i}.fbx")).unwrap().temp_download_url.clone(),
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
                            Some(format!("{base_path}/{}", map.get("uri").unwrap().as_str().unwrap()))
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
                                        println!("format={}", format.get("mimeType").unwrap().as_str().unwrap());
                                        println!("uri={}", format.get("uri").unwrap().as_str().unwrap());
                                        if format.get("mimeType").unwrap().as_str().unwrap() == "image/jpeg"
                                            && format
                                                .get("uri")
                                                .unwrap()
                                                .as_str()
                                                .unwrap()
                                                .contains(&format!("_{}_", quixel_id.resolution.to_uppercase()))
                                        {
                                            return Some(format!("{base_path}/{}", format.get("uri").unwrap().as_str().unwrap()));
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

            let atlas = PbrMaterialFromUrl {
                base_color: pipe_image(ctx, materials, get_map_v1("maps", "Albedo").or_else(|| get_map_v2("components", "Albedo")), |_| {})
                    .await?
                    .map(|x| material_image_path(materials_id, x)),
                opacity: pipe_image(ctx, materials, get_map_v1("maps", "Opacity").or_else(|| get_map_v2("components", "Opacity")), |_| {})
                    .await?
                    .map(|x| material_image_path(materials_id, x)),
                normalmap: pipe_image(ctx, materials, get_map_v1("maps", "Normal").or_else(|| get_map_v2("components", "Normal")), |_| {})
                    .await?
                    .map(|x| material_image_path(materials_id, x)),
                alpha_cutoff: Some(0.5),
                metallic_roughness: pipe_image(
                    ctx,
                    materials,
                    get_map_v1("maps", "Roughness").or_else(|| get_map_v2("components", "Roughness")),
                    rougness_to_mr,
                )
                .await?
                .map(|x| material_image_path(materials_id, x)),
                metallic: 0.2,
                roughness: 1.0,
                ..Default::default()
            };
            let billboard = PbrMaterialFromUrl {
                base_color: pipe_image(ctx, materials, get_map_v1("billboards", "Albedo"), |_| {})
                    .await?
                    .map(|x| material_image_path(materials_id, x)),
                opacity: pipe_image(ctx, materials, get_map_v1("billboards", "Opacity"), |_| {})
                    .await?
                    .map(|x| material_image_path(materials_id, x)),
                normalmap: pipe_image(ctx, materials, get_map_v1("billboards", "Normal"), |_| {})
                    .await?
                    .map(|x| material_image_path(materials_id, x)),
                alpha_cutoff: Some(0.5),
                metallic_roughness: pipe_image(ctx, materials, get_map_v1("maps", "Roughness"), rougness_to_mr)
                    .await?
                    .map(|x| material_image_path(materials_id, x)),
                metallic: 0.2,
                roughness: 1.0,
                ..Default::default()
            };
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
                            let file = format!("{base_path}/Var{variation_nr}/Var{variation_nr}_LOD{lod_index}.fbx");
                            lods.push(
                                ModelImportPipeline::new()
                                    .add_step(ModelImportTransform::ImportModelFromUrl {
                                        url: ctx.files.get(&file).unwrap().temp_download_url.clone(),
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
                    if !ctx.files.contains_key(&format!("{base_path}/Var{var}/Var{var}_LOD{lod}.fbx", var = variation, lod = 0)) {
                        break;
                    }
                    let lods = (0..6)
                        .map(|i| {
                            ModelImportPipeline::new().add_step(ModelImportTransform::ImportModelFromUrl {
                                url: ctx
                                    .files
                                    .get(&format!("{base_path}/Var{variation}/Var{variation}_LOD{i}.fbx"))
                                    .unwrap()
                                    .temp_download_url
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
    pub fn from_full(full: &str) -> Option<Self> {
        // Props_Storage_vijncb3_2K_3d_ms
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
