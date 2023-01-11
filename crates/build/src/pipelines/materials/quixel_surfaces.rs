use std::sync::Arc;

use convert_case::{Case, Casing};
use elements_model_import::{dotdot_path, model_crate::ModelCrate};
use elements_renderer::materials::pbr_material::PbrMaterialFromUrl;
use elements_std::{
    asset_cache::AssetCache, asset_url::{AbsAssetUrl, AssetType}
};
use futures::future::join_all;
use itertools::Itertools;

use super::{
    super::{models::quixel::QuixelId, OutAsset, OutAssetContent, OutAssetPreview, PipelineCtx}, MaterialsPipeline
};
use crate::helpers::download_json;

pub async fn pipeline(ctx: &PipelineCtx, _config: MaterialsPipeline) -> Vec<OutAsset> {
    ctx.process_files(
        |file| {
            file.sub_path.extension == Some("json".to_string()) && file.sub_path_string.contains(&format!("_{}_", file.sub_path.filename))
        },
        move |ctx, file| async move {
            let mut res = Vec::new();
            let quixel_id = QuixelId::from_full(file.sub_path.path.last().unwrap()).unwrap();
            let quixel_json: serde_json::Value = download_json(&ctx.assets, &file.temp_download_url).await.unwrap();
            let base_path = file.sub_path.path.join("/");
            let surface = QuixelSurfaceDef::from_quixel_json(&ctx, &quixel_id, &quixel_json, &base_path);
            let asset_crate_id = ctx.asset_crate_id(&file.sub_path_string);
            let asset_crate_url = ctx.crate_url(&asset_crate_id);
            let mut asset_crate = ModelCrate::new();
            surface.write_to_asset_crate(&ctx.assets, &mut asset_crate).await;

            let tags =
                quixel_json["tags"].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string().to_case(Case::Title)).collect_vec();
            let pack_name = quixel_json["semanticTags"]["name"].as_str().unwrap().to_string();

            res.push(OutAsset {
                asset_crate_id: asset_crate_id.clone(),
                sub_asset: None,
                type_: AssetType::Material,
                hidden: false,
                name: pack_name.clone(),
                tags,
                categories: Default::default(),
                preview: OutAssetPreview::Image { image: Arc::new(asset_crate.images.content.get("base_color").unwrap().clone()) },
                content: OutAssetContent::Content(
                    asset_crate_url.resolve(asset_crate.materials.loc.path(ModelCrate::MAIN).as_str()).unwrap(),
                ),
                source: Some(file.sub_path_string.clone()),
            });
            ctx.write_model_crate(&asset_crate, &asset_crate_id).await;
            Ok(res)
        },
    )
    .await
}
async fn download_image(assets: &AssetCache, url: Option<AbsAssetUrl>) -> Option<image::RgbaImage> {
    if let Some(url) = url {
        Some(crate::helpers::download_image(assets, &url, &None).await.ok()?.into_rgba8())
    } else {
        None
    }
}

#[derive(Clone, Debug, Default)]
pub struct QuixelSurfaceDef {
    pub albedo: Option<AbsAssetUrl>,
    pub ao: Option<AbsAssetUrl>,
    pub normal: Option<AbsAssetUrl>,
}
impl QuixelSurfaceDef {
    async fn write_to_asset_crate(&self, assets: &AssetCache, asset_crate: &mut ModelCrate) {
        let mut images = join_all(
            [
                download_image(assets, self.albedo.clone()),
                download_image(assets, self.ao.clone()),
                download_image(assets, self.normal.clone()),
            ]
            .into_iter(),
        )
        .await;
        let mut albedo = images.remove(0);
        let ao = images.remove(0);
        let normal = images.remove(0);
        if let (Some(albedo), Some(ao)) = (&mut albedo, &ao) {
            // Pre-multiply AO
            for (b, ao) in albedo.pixels_mut().zip(ao.pixels()) {
                b[0] = ((ao[0] as f32 / 255.) * b[0] as f32) as u8;
                b[1] = ((ao[0] as f32 / 255.) * b[1] as f32) as u8;
                b[2] = ((ao[0] as f32 / 255.) * b[2] as f32) as u8;
            }
        }
        let mat = PbrMaterialFromUrl {
            base_color: albedo.map(|albedo| asset_crate.images.insert("base_color", albedo).path).map(|x| dotdot_path(x).into()),
            normalmap: normal.map(|normal| asset_crate.images.insert("normalmap", normal).path).map(|x| dotdot_path(x).into()),
            ..Default::default()
        };
        asset_crate.materials.insert(ModelCrate::MAIN, mat);
    }
    fn from_quixel_json(ctx: &PipelineCtx, qid: &QuixelId, json: &serde_json::Value, base_path: &str) -> Self {
        let mut res = Self::default();
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
                        if resolution["resolution"].as_str().unwrap().starts_with(target_resolution) {
                            for format in resolution["formats"].as_array().unwrap() {
                                if format["mimeType"].as_str().unwrap() == "image/jpeg" {
                                    let path = format!("{base_path}/{}", format["uri"].as_str().unwrap());
                                    if let Some(url) = ctx.files.get(&path).map(|x| x.temp_download_url.clone()) {
                                        match comp_type {
                                            "albedo" => res.albedo = Some(url),
                                            "ao" => res.ao = Some(url),
                                            "normal" => res.normal = Some(url),
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
                if map["mimeType"].as_str().unwrap() == "image/jpeg" && map["resolution"].as_str().unwrap().starts_with(target_resolution) {
                    let path = format!("{base_path}/{}", map["uri"].as_str().unwrap());
                    if let Some(url) = ctx.files.get(&path).map(|x| x.temp_download_url.clone()) {
                        match map["type"].as_str().unwrap() {
                            "albedo" => res.albedo = Some(url),
                            "ao" => res.ao = Some(url),
                            "normal" => res.normal = Some(url),
                            _ => {}
                        }
                    }
                }
            }
        }
        res
    }
}
