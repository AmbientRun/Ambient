use std::sync::Arc;

use elements_model_import::{model_crate::ModelCrate, RelativePathBufExt};
use elements_renderer::materials::pbr_material::PbrMaterialFromUrl;
use elements_std::asset_url::AssetType;
use glam::Vec4;
use image::RgbaImage;
use relative_path::RelativePathBuf;
use serde::{Deserialize, Serialize};

use super::{
    context::PipelineCtx, out_asset::{OutAsset, OutAssetContent, OutAssetPreview}
};
use crate::helpers::download_image;

pub mod quixel_surfaces;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[allow(clippy::large_enum_variant)]
pub enum MaterialsImporter {
    Single(PipelinePbrMaterial),
    Quixel,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialsPipeline {
    pub importer: MaterialsImporter,
}

pub async fn pipeline(ctx: &PipelineCtx, config: MaterialsPipeline) -> Vec<OutAsset> {
    match config.importer {
        MaterialsImporter::Single(mat) => {
            ctx.process_single(move |ctx| async move {
                let name = mat.name.as_ref().or(mat.source.as_ref()).or(mat.base_color.as_ref()).unwrap().to_string();
                let asset_crate_id = ctx.asset_crate_id(&name);
                let asset_crate_url = ctx.crate_url(&asset_crate_id);
                let mut model_crate = ModelCrate::new();
                let material = mat.to_mat(&ctx, &mut model_crate).await?;
                let mat_url =
                    asset_crate_url.resolve(model_crate.materials.insert(ModelCrate::MAIN, material.clone()).path.as_str()).unwrap();
                ctx.write_model_crate(&model_crate, &asset_crate_id).await;
                Ok(vec![OutAsset {
                    asset_crate_id: asset_crate_id.clone(),
                    sub_asset: None,
                    type_: AssetType::Material,
                    hidden: false,
                    name,
                    tags: Default::default(),
                    categories: Default::default(),
                    preview: OutAssetPreview::Image {
                        image: Arc::new(model_crate.images.get_by_path(material.base_color.as_ref().unwrap().path()).unwrap().clone()),
                    },
                    content: OutAssetContent::Content(mat_url),
                    source: None,
                }])
            })
            .await
        }
        MaterialsImporter::Quixel => quixel_surfaces::pipeline(ctx, config).await,
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct PipelinePbrMaterial {
    pub name: Option<String>,
    pub source: Option<String>,

    pub base_color: Option<String>,
    pub opacity: Option<String>,
    pub normalmap: Option<String>,
    pub metallic_roughness: Option<String>,

    pub base_color_factor: Option<Vec4>,
    pub emissive_factor: Option<Vec4>,
    pub transparent: Option<bool>,
    pub alpha_cutoff: Option<f32>,
    pub double_sided: Option<bool>,
    pub metallic: Option<f32>,
    pub roughness: Option<f32>,

    // Non-pbr properties that gets translated to pbr
    pub specular: Option<String>,
    pub specular_exponent: Option<f32>,
}
impl PipelinePbrMaterial {
    pub async fn to_mat(&self, ctx: &PipelineCtx, model_crate: &mut ModelCrate) -> anyhow::Result<PbrMaterialFromUrl> {
        Ok(PbrMaterialFromUrl {
            name: self.name.clone(),
            source: self.source.clone(),
            base_color: pipe_image(ctx, model_crate, self.base_color.clone(), |_| {}).await?.map(|x| x.prejoin("..").into()),
            opacity: pipe_image(ctx, model_crate, self.opacity.clone(), |_| {}).await?.map(|x| x.prejoin("..").into()),
            normalmap: pipe_image(ctx, model_crate, self.normalmap.clone(), |_| {}).await?.map(|x| x.prejoin("..").into()),
            metallic_roughness: if let Some(url) = pipe_image(ctx, model_crate, self.metallic_roughness.clone(), |_| {}).await? {
                Some(url.prejoin("..").into())
            } else {
                pipe_image(ctx, model_crate, self.specular.clone(), |image| {
                    for p in image.pixels_mut() {
                        let specular = 1. - (1. - p[1] as f32 / 255.).powf(self.specular_exponent.unwrap_or(1.));
                        p[0] = (specular * 255.) as u8;
                        p[1] = ((1. - specular) * 255.) as u8;
                        p[2] = 0;
                        p[3] = 255;
                    }
                })
                .await?
                .map(|x| x.prejoin("..").into())
            },

            base_color_factor: self.base_color_factor,
            emissive_factor: self.emissive_factor,
            transparent: self.transparent,
            alpha_cutoff: self.alpha_cutoff,
            double_sided: self.double_sided,
            metallic: self.metallic.unwrap_or(1.),
            roughness: self.roughness.unwrap_or(1.),
        })
    }
}

/// Reads an image from the asset pack, writes it to the model_crate, and optionally processes it in between
pub async fn pipe_image(
    ctx: &PipelineCtx,
    model_crate: &mut ModelCrate,
    path: Option<String>,
    process: impl Fn(&mut RgbaImage) + Sync + Send,
) -> anyhow::Result<Option<RelativePathBuf>> {
    if let Some(path) = path {
        let file = ctx.get_file(&path)?;
        let mut image = download_image(&ctx.assets, &file.temp_download_url, &file.sub_path.extension).await.unwrap().into_rgba8();
        process(&mut image);
        let path = format!("{}/{}", file.sub_path.path.join("/"), file.sub_path.filename);
        Ok(Some(model_crate.images.insert(path, image).path))
    } else {
        Ok(None)
    }
}
