use std::sync::Arc;

use ambient_model_import::{dotdot_path, model_crate::ModelCrate};
use ambient_native_std::{
    asset_cache::AssetCache,
    asset_url::{AbsAssetUrl, AssetType},
};
use ambient_pipeline_types::materials::{MaterialsPipeline, QuixelSurfaceDef};
use ambient_renderer::materials::pbr_material::PbrMaterialDesc;
use convert_case::{Case, Casing};
use futures::future::join_all;
use itertools::Itertools;

use super::super::{
    models::quixel::QuixelId, OutAsset, OutAssetContent, OutAssetPreview, PipelineCtx,
};
use crate::pipelines::out_asset::asset_id_from_url;

pub async fn pipeline(ctx: &PipelineCtx, _config: MaterialsPipeline) -> Vec<OutAsset> {
    ctx.process_files(
        |file| {
            file.extension() == Some("json".to_string())
                && file
                    .decoded_path()
                    .to_string()
                    .contains(&format!("_{}_", file.decoded_path().file_stem().unwrap()))
        },
        move |ctx, file| async move {
            let mut res = Vec::new();
            let quixel_id = QuixelId::from_full(file.last_dir_name().unwrap()).unwrap();
            let quixel_json: serde_json::Value = file.download_json(ctx.assets()).await.unwrap();
            let in_root_url = file.join(".").unwrap();
            let surface = from_quixel_json(&ctx, &quixel_id, &quixel_json, &in_root_url);

            let mut asset_crate = ModelCrate::new();

            write_to_asset_crate(&surface, ctx.assets(), &mut asset_crate).await;

            let tags = quixel_json["tags"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_str().unwrap().to_string().to_case(Case::Title))
                .collect_vec();

            let pack_name = quixel_json["semanticTags"]["name"]
                .as_str()
                .unwrap()
                .to_string();

            let model_crate_url = ctx
                .write_model_crate(
                    &asset_crate,
                    &ctx.in_root().relative_path(file.decoded_path()),
                )
                .await;

            res.push(OutAsset {
                id: asset_id_from_url(&file),
                type_: AssetType::Material,
                hidden: false,
                name: pack_name.clone(),
                tags,
                categories: Default::default(),
                preview: asset_crate
                    .images
                    .content
                    .get("base_color")
                    .or(asset_crate.images.content.get("opacity"))
                    .or(asset_crate.images.content.get("normal"))
                    .map(|image| OutAssetPreview::Image {
                        image: Arc::new(image.clone()),
                    })
                    .unwrap_or(OutAssetPreview::None),
                content: OutAssetContent::Content(
                    model_crate_url.material(ModelCrate::MAIN).abs().unwrap(),
                ),
                source: Some(file.clone()),
            });
            Ok(res)
        },
    )
    .await
}

async fn download_image(assets: &AssetCache, url: Option<AbsAssetUrl>) -> Option<image::RgbaImage> {
    if let Some(url) = url {
        Some(super::download_image(assets, &url).await.ok()?.into_rgba8())
    } else {
        None
    }
}

async fn write_to_asset_crate(
    surface: &QuixelSurfaceDef,
    assets: &AssetCache,
    asset_crate: &mut ModelCrate,
) {
    let mut images = join_all(
        [
            download_image(assets, surface.albedo.clone()),
            download_image(assets, surface.ao.clone()),
            download_image(assets, surface.normal.clone()),
            download_image(assets, surface.opacity.clone()),
        ]
        .into_iter(),
    )
    .await;

    let mut albedo = images.remove(0);
    let ao = images.remove(0);
    let normal = images.remove(0);
    let opacity = images.remove(0);
    let transparent = Some(opacity.is_some());

    if let (Some(albedo), Some(ao)) = (&mut albedo, &ao) {
        // Pre-multiply AO
        for (b, ao) in albedo.pixels_mut().zip(ao.pixels()) {
            b[0] = ((ao[0] as f32 / 255.) * b[0] as f32) as u8;
            b[1] = ((ao[0] as f32 / 255.) * b[1] as f32) as u8;
            b[2] = ((ao[0] as f32 / 255.) * b[2] as f32) as u8;
        }
    }

    if let (Some(albedo), Some(opacity)) = (&mut albedo, &opacity) {
        for (b, op) in albedo.pixels_mut().zip(opacity.pixels()) {
            b[3] = op[0];
        }
    }

    let mat = PbrMaterialDesc {
        transparent,
        base_color: albedo
            .map(|albedo| asset_crate.images.insert("base_color", albedo).path)
            .map(|x| dotdot_path(x).into()),
        normalmap: normal
            .map(|normal| asset_crate.images.insert("normalmap", normal).path)
            .map(|x| dotdot_path(x).into()),
        ..Default::default()
    };

    asset_crate.materials.insert(ModelCrate::MAIN, mat);
}
fn from_quixel_json(
    ctx: &PipelineCtx,
    qid: &QuixelId,
    json: &serde_json::Value,
    in_root_url: &AbsAssetUrl,
) -> QuixelSurfaceDef {
    let mut res = QuixelSurfaceDef::default();

    let target_resolution = match &qid.resolution as &str {
        "1K" => "1024x",
        "2K" => "2048x",
        "4K" => "4096x",
        "8K" => "8192x",
        _ => panic!("Unsupported resolution: {:?}", qid.resolution),
    };

    if let Some(components) = json["components"].as_array() {
        for comp in components {
            let comp_type = comp["type"].as_str().unwrap();
            for uri in comp["uris"].as_array().unwrap() {
                for resolution in uri["resolutions"].as_array().unwrap() {
                    if resolution["resolution"]
                        .as_str()
                        .unwrap()
                        .starts_with(target_resolution)
                    {
                        for format in resolution["formats"].as_array().unwrap() {
                            if format["mimeType"].as_str().unwrap() == "image/jpeg" {
                                if let Ok(url) = ctx.get_downloadable_url(
                                    &in_root_url.push(format["uri"].as_str().unwrap()).unwrap(),
                                ) {
                                    match comp_type {
                                        "albedo" => res.albedo = Some(url.clone()),
                                        "ao" => res.ao = Some(url.clone()),
                                        "normal" => res.normal = Some(url.clone()),
                                        "opacity" => res.opacity = Some(url.clone()),
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        for map in json["maps"].as_array().unwrap() {
            if map["mimeType"].as_str().unwrap() == "image/jpeg"
                && map["resolution"]
                    .as_str()
                    .unwrap()
                    .starts_with(target_resolution)
            {
                if let Ok(url) = ctx
                    .get_downloadable_url(&in_root_url.push(map["uri"].as_str().unwrap()).unwrap())
                {
                    match map["type"].as_str().unwrap() {
                        "albedo" => res.albedo = Some(url.clone()),
                        "ao" => res.ao = Some(url.clone()),
                        "normal" => res.normal = Some(url.clone()),
                        "opacity" => res.opacity = Some(url.clone()),
                        _ => {}
                    }
                }
            }
        }
    }

    res
}
