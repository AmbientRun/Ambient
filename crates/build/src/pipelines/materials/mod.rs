use std::{io::Cursor, sync::Arc};

use anyhow::Context;
use async_trait::async_trait;
use dyn_clonable::*;
use elements_asset_cache::{AssetCache, AssetKeepalive, AsyncAssetKey, AsyncAssetKeyExt, SyncAssetKeyExt};
use elements_core::transform;
use elements_model_import::{
    model_crate::{cap_texture_size, ModelCrate}, ModelTextureSize, RelativePathBufExt
};
use elements_renderer::materials::pbr_material::PbrMaterialFromUrl;
use elements_std::{
    asset_url::{AbsAssetUrl, AssetType, AssetUrl}, download_asset::AssetResult
};
use glam::Vec4;
use image::{ImageOutputFormat, RgbaImage};
use relative_path::{RelativePath, RelativePathBuf};
use serde::{Deserialize, Serialize};

use super::{
    context::PipelineCtx, out_asset::{OutAsset, OutAssetContent, OutAssetPreview}, ProcessCtxKey
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
                let mat_out_url = ctx.out_root().join("material")?.join("materials")?;
                let material = mat.to_mat(&ctx, &ctx.in_root(), &mat_out_url).await?;
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
    pub async fn to_mat(&self, ctx: &PipelineCtx, source_root: &AbsAssetUrl, out_root: &AbsAssetUrl) -> anyhow::Result<PbrMaterialFromUrl> {
        Ok(PbrMaterialFromUrl {
            name: self.name.clone(),
            source: self.source.clone(),
            base_color: PipeImage::resolve_opt(ctx.assets(), &source_root, &self.base_color, None, &out_root).await?,
            opacity: PipeImage::resolve_opt(ctx.assets(), &source_root, &self.opacity, None, &out_root).await?,
            normalmap: PipeImage::resolve_opt(ctx.assets(), &source_root, &self.normalmap, None, &out_root).await?,
            metallic_roughness: if let Some(url) =
                PipeImage::resolve_opt(ctx.assets(), &source_root, &self.metallic_roughness, None, &out_root).await?
            {
                Some(url)
            } else {
                let specular_exponent = self.specular_exponent.unwrap_or(1.);
                let transform = FnImageTransformer::new("mr_from_s", move |image, _| {
                    for p in image.pixels_mut() {
                        let specular = 1. - (1. - p[1] as f32 / 255.).powf(specular_exponent);
                        p[0] = (specular * 255.) as u8;
                        p[1] = ((1. - specular) * 255.) as u8;
                        p[2] = 0;
                        p[3] = 255;
                    }
                });
                PipeImage::resolve_opt(ctx.assets(), &source_root, &self.specular, Some(transform), &out_root).await?
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

#[clonable]
pub trait ImageTransformer: std::fmt::Debug + Clone + Sync + Send {
    fn transform(&self, image: &mut RgbaImage, second_image: Option<&RgbaImage>);
    fn name(&self) -> &str;
}
pub struct FnImageTransformer<F: Fn(&mut RgbaImage, Option<&RgbaImage>) + Sync + Send + 'static> {
    func: Arc<F>,
    name: &'static str,
}
impl<F: Fn(&mut RgbaImage, Option<&RgbaImage>) + Sync + Send + 'static> FnImageTransformer<F> {
    pub fn new(name: &'static str, func: F) -> Box<dyn ImageTransformer> {
        Box::new(Self { func: Arc::new(func), name })
    }
}
impl<F: Fn(&mut RgbaImage, Option<&RgbaImage>) + Sync + Send + 'static> Clone for FnImageTransformer<F> {
    fn clone(&self) -> Self {
        Self { func: self.func.clone(), name: self.name.clone() }
    }
}
impl<F: Fn(&mut RgbaImage, Option<&RgbaImage>) + Sync + Send + 'static> std::fmt::Debug for FnImageTransformer<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FnImageTransformer").field("name", &self.name).finish()
    }
}
impl<F: Fn(&mut RgbaImage, Option<&RgbaImage>) + Sync + Send + 'static> ImageTransformer for FnImageTransformer<F> {
    fn transform(&self, image: &mut RgbaImage, second_image: Option<&RgbaImage>) {
        (self.func)(image, second_image)
    }
    fn name(&self) -> &str {
        self.name
    }
}

#[derive(Debug, Clone)]
pub struct PipeImage {
    source: AbsAssetUrl,
    second_source: Option<AbsAssetUrl>,
    transform: Option<Box<dyn ImageTransformer>>,
    cap_texture_sizes: Option<ModelTextureSize>,
}
impl PipeImage {
    pub async fn resolve_opt(
        assets: &AssetCache,
        source_root: &AbsAssetUrl,
        source: &Option<AssetUrl>,
        transform: Option<Box<dyn ImageTransformer>>,
        base_url: &AbsAssetUrl,
    ) -> anyhow::Result<Option<AssetUrl>> {
        if let Some(source) = source {
            Ok(Some(
                PipeImage::resolve(assets, source.resolve(source_root).context("Failed to resolve root")?, transform, base_url)
                    .await?
                    .into(),
            ))
        } else {
            Ok(None)
        }
    }
    pub async fn resolve(
        assets: &AssetCache,
        source: AbsAssetUrl,
        transform: Option<Box<dyn ImageTransformer>>,
        base_url: &AbsAssetUrl,
    ) -> anyhow::Result<RelativePathBuf> {
        let url = PipeImage { source, second_source: None, transform, cap_texture_sizes: None }.get(assets).await?;
        Ok(base_url.relative_path(url.path()))
    }
}
#[async_trait]
impl AsyncAssetKey<AssetResult<Arc<AbsAssetUrl>>> for PipeImage {
    async fn load(self, assets: AssetCache) -> AssetResult<Arc<AbsAssetUrl>> {
        let ctx = ProcessCtxKey.get(&assets);
        let mut image = (*ImageFromUrl { url: self.source.clone() }
            .get(&assets)
            .await
            .with_context(|| format!("Failed to download image {}", self.source))?)
        .clone();
        let mut extension = "png".to_string();
        let second_image = if let Some(second_source) = &self.second_source {
            Some(
                ImageFromUrl { url: second_source.clone() }
                    .get(&assets)
                    .await
                    .with_context(|| format!("Failed to download second image {}", self.source))?,
            )
        } else {
            None
        };
        let path = ctx.in_root.relative_path(&self.source.path());
        let mut data = Cursor::new(Vec::new());
        tokio::task::block_in_place(|| {
            if let Some(transform) = &self.transform {
                transform.transform(&mut image, second_image.as_ref().map(|x| &**x));
                extension = format!("{}.png", transform.name());
            }
            if let Some(size) = self.cap_texture_sizes {
                cap_texture_size(&mut image, size.size());
            }
            image.write_to(&mut data, ImageOutputFormat::Png).unwrap();
        });
        Ok(Arc::new((ctx.write_file)(path.with_extension(extension).to_string(), data.into_inner()).await))
    }
}

#[derive(Debug, Clone)]
pub struct ImageFromUrl {
    pub url: AbsAssetUrl,
}
#[async_trait]
impl AsyncAssetKey<AssetResult<Arc<image::RgbaImage>>> for ImageFromUrl {
    fn keepalive(&self) -> AssetKeepalive {
        AssetKeepalive::None
    }
    async fn load(self, assets: AssetCache) -> AssetResult<Arc<image::RgbaImage>> {
        Ok(Arc::new(download_image(&assets, &self.url).await?.into_rgba8()))
    }
}
