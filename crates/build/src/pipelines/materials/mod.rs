use std::sync::Arc;

use elements_model_import::{model_crate::ModelCrate, RelativePathBufExt};
use elements_renderer::materials::pbr_material::PbrMaterialFromUrl;
use elements_std::asset_url::{AbsAssetUrl, AssetType, AssetUrl};
use glam::Vec4;
use image::RgbaImage;
use relative_path::{RelativePath, RelativePathBuf};
use serde::{Deserialize, Serialize};

use super::{
    context::PipelineCtx, out_asset::{OutAsset, OutAssetContent, OutAssetPreview}
};
use crate::pipelines::download_image;

// pub mod quixel_surfaces;

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
                let name = mat.name.as_ref().or(mat.source.as_ref()).unwrap().to_string();

                let mut model_crate = ModelCrate::new();
                let material = mat.to_mat(&ctx, &mut model_crate, ctx.in_root().clone()).await?;
                model_crate.materials.insert(ModelCrate::MAIN, material);
                let model_url = ctx.write_model_crate(&model_crate, &RelativePath::new("material")).await;
                Ok(vec![OutAsset {
                    id: ctx.in_root().to_string(),
                    type_: AssetType::Material,
                    hidden: false,
                    name,
                    tags: Default::default(),
                    categories: Default::default(),
                    preview: OutAssetPreview::Image { image: Arc::new(model_crate.images.content.get("base_color").unwrap().clone()) },
                    content: OutAssetContent::Content(model_url.model_crate().unwrap().material(ModelCrate::MAIN).abs().unwrap()),
                    source: None,
                }])
            })
            .await
        }
        MaterialsImporter::Quixel => {
            todo!()
            // quixel_surfaces::pipeline(ctx, config).await
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct PipelinePbrMaterial {
    pub name: Option<String>,
    pub source: Option<String>,

    pub base_color: Option<AssetUrl>,
    pub opacity: Option<AssetUrl>,
    pub normalmap: Option<AssetUrl>,
    pub metallic_roughness: Option<AssetUrl>,

    pub base_color_factor: Option<Vec4>,
    pub emissive_factor: Option<Vec4>,
    pub transparent: Option<bool>,
    pub alpha_cutoff: Option<f32>,
    pub double_sided: Option<bool>,
    pub metallic: Option<f32>,
    pub roughness: Option<f32>,

    // Non-pbr properties that gets translated to pbr
    pub specular: Option<AssetUrl>,
    pub specular_exponent: Option<f32>,
}
impl PipelinePbrMaterial {
    pub async fn to_mat(
        &self,
        ctx: &PipelineCtx,
        model_crate: &mut ModelCrate,
        self_url: AbsAssetUrl,
    ) -> anyhow::Result<PbrMaterialFromUrl> {
        /// Reads an image from the asset pack, writes it to the model_crate, and optionally processes it in between
        async fn pipe_image(
            name: &str,
            ctx: &PipelineCtx,
            model_crate: &mut ModelCrate,
            self_url: &AbsAssetUrl,
            source_url: Option<AssetUrl>,
            process: impl Fn(&mut RgbaImage) + Sync + Send,
        ) -> anyhow::Result<Option<AssetUrl>> {
            if let Some(source_url) = source_url {
                let url = source_url.resolve(self_url).unwrap();
                let mut image = download_image(&ctx.process_ctx.assets, &url).await.unwrap().into_rgba8();
                process(&mut image);
                model_crate.images.insert(name.to_string(), image);
                Ok(Some(AssetUrl::Relative(format!("../images/{name}").into())))
            } else {
                Ok(None)
            }
        }

        Ok(PbrMaterialFromUrl {
            name: self.name.clone(),
            source: self.source.clone(),
            base_color: pipe_image("base_color", ctx, model_crate, &self_url, self.base_color.clone(), |_| {}).await?,
            opacity: pipe_image("opacity", ctx, model_crate, &self_url, self.opacity.clone(), |_| {}).await?,
            normalmap: pipe_image("normalmap", ctx, model_crate, &self_url, self.normalmap.clone(), |_| {}).await?,
            metallic_roughness: if let Some(url) =
                pipe_image("metallic_roughness", ctx, model_crate, &self_url, self.metallic_roughness.clone(), |_| {}).await?
            {
                Some(url)
            } else {
                pipe_image("metallic_roughness", ctx, model_crate, &self_url, self.specular.clone(), |image| {
                    for p in image.pixels_mut() {
                        let specular = 1. - (1. - p[1] as f32 / 255.).powf(self.specular_exponent.unwrap_or(1.));
                        p[0] = (specular * 255.) as u8;
                        p[1] = ((1. - specular) * 255.) as u8;
                        p[2] = 0;
                        p[3] = 255;
                    }
                })
                .await?
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
