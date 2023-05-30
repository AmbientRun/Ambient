use std::{io::Cursor, sync::Arc};

use ambient_asset_cache::{
    AssetCache, AssetKeepalive, AsyncAssetKey, AsyncAssetKeyExt, SyncAssetKeyExt,
};
use ambient_decals::decal;
use ambient_ecs::Entity;
use ambient_gpu::sampler::SamplerKey;
use ambient_model_import::{
    model_crate::{cap_texture_size, ModelCrate},
    ModelTextureSize,
};
use ambient_physics::collider::{collider, collider_type};
use ambient_renderer::materials::pbr_material::PbrMaterialDesc;
use ambient_std::{
    asset_url::{AbsAssetUrl, AssetType, AssetUrl},
    download_asset::AssetResult,
};
use anyhow::Context;
use async_trait::async_trait;
use dyn_clonable::*;
use futures::{future::BoxFuture, FutureExt};
use glam::{Vec3, Vec4};
use image::{ImageOutputFormat, RgbaImage};
use serde::{Deserialize, Serialize};

use super::{
    context::PipelineCtx,
    out_asset::{asset_id_from_url, OutAsset, OutAssetContent, OutAssetPreview},
    ProcessCtxKey,
};
use crate::pipelines::download_image;

pub mod quixel_surfaces;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[allow(clippy::large_enum_variant)]
#[serde(deny_unknown_fields)]
pub enum MaterialsImporter {
    /// Import a single material, as specified.
    /// All of its dependent assets (URLs, etc) will be resolved during the build process.
    Single(PipelinePbrMaterial),
    /// Import Quixel materials.
    Quixel,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialsPipeline {
    /// The importer to use for materials.
    pub importer: Box<MaterialsImporter>,
    /// Whether or not decal prefabs should be created for each of these materials.
    #[serde(default)]
    pub output_decals: bool,
}

pub async fn pipeline(ctx: &PipelineCtx, config: MaterialsPipeline) -> Vec<OutAsset> {
    let materials = match *config.importer.clone() {
        MaterialsImporter::Single(mat) => {
            ctx.process_single(move |ctx| async move {
                let name = mat
                    .name
                    .as_ref()
                    .or(mat.source.as_ref())
                    .unwrap()
                    .to_string();

                let mat_out_url = ctx.out_root().join(ctx.pipeline_path())?.as_directory();
                let material = mat.to_mat(&ctx, &ctx.in_root(), &mat_out_url).await?;
                let base_color_url = material
                    .base_color
                    .clone()
                    .unwrap()
                    .resolve(&mat_out_url)
                    .unwrap();
                let base_color = ImageFromUrl {
                    url: base_color_url,
                }
                .get(ctx.assets())
                .await?;
                let mat_url = ctx
                    .write_file(
                        ctx.pipeline_path().join("mat.json"),
                        serde_json::to_vec(&material).unwrap(),
                    )
                    .await;
                Ok(vec![OutAsset {
                    id: asset_id_from_url(&ctx.out_root()),
                    type_: AssetType::Material,
                    hidden: false,
                    name,
                    tags: Default::default(),
                    categories: Default::default(),
                    preview: OutAssetPreview::Image { image: base_color },
                    content: OutAssetContent::Content(mat_url),
                    source: None,
                }])
            })
            .await
        }
        MaterialsImporter::Quixel => quixel_surfaces::pipeline(ctx, config.clone()).await,
    };
    if config.output_decals {
        let mut res = materials.clone();
        for mat in materials {
            if let OutAssetContent::Content(mat_url) = mat.content {
                let model_path = ctx
                    .in_root()
                    .relative_path(
                        mat.source
                            .clone()
                            .map(|x| x.decoded_path())
                            .unwrap_or_else(|| ctx.pipeline_path()),
                    )
                    .join("decal");
                let out_model_url = ctx.out_root().join(&model_path).unwrap();
                let mut model_crate = ModelCrate::new();
                let decal_path = out_model_url
                    .decoded_path()
                    .join("prefabs")
                    .relative(mat_url.decoded_path());
                model_crate.create_prefab(
                    Entity::new()
                        .with(decal(), decal_path.into())
                        .with(
                            collider(),
                            ambient_physics::collider::ColliderDef::Box {
                                size: Vec3::ONE,
                                center: Vec3::ZERO,
                            },
                        )
                        .with(
                            collider_type(),
                            ambient_physics::collider::ColliderType::Picking,
                        ),
                );
                let model_url = ctx.write_model_crate(&model_crate, &model_path).await;
                res.push(OutAsset {
                    id: asset_id_from_url(&out_model_url),
                    type_: AssetType::Prefab,
                    hidden: false,
                    name: mat.name,
                    tags: mat.tags,
                    categories: mat.categories,
                    preview: mat.preview,
                    content: OutAssetContent::Content(model_url.prefab().unwrap_abs()),
                    source: mat.source,
                });
            }
        }
        res
    } else {
        materials
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct PipelinePbrMaterial {
    /// The name of the material.
    pub name: Option<String>,
    /// Where the material came from.
    pub source: Option<String>,

    /// The base color map (i.e. texture) of this material.
    pub base_color: Option<AssetUrl>,
    /// The opacity map of this material.
    pub opacity: Option<AssetUrl>,
    /// The normal map of this material.
    pub normalmap: Option<AssetUrl>,
    /// The metallic roughness map of this material.
    pub metallic_roughness: Option<AssetUrl>,

    /// The color that this material should be multiplied by. Defaults to white for PBR.
    pub base_color_factor: Option<Vec4>,
    /// The emissive factor of this material (i.e. the color that it emits). Defaults to black for PBR.
    pub emissive_factor: Option<Vec4>,
    /// Whether or not this material is transparent. Defaults to false for PBR.
    pub transparent: Option<bool>,
    /// The opacity level (between 0 and 1) at which this material will not be rendered.
    /// If the opacity map at a point has an opacity lower than this, that point will not be rendered.
    /// Defaults to 0.5 for PBR.
    pub alpha_cutoff: Option<f32>,
    /// Whether or not this material is double-sided. Defaults to false for PBR.
    pub double_sided: Option<bool>,
    /// The metallic coefficient of this material. Defaults to 1 for PBR.
    pub metallic: Option<f32>,
    /// The roughness coefficient of this material. Defaults to 1 for PBR.
    pub roughness: Option<f32>,

    // Non-PBR properties that get translated to PBR.
    /// The non-PBR specular map of this material. If specified, it will be translated to a PBR equivalent.
    pub specular: Option<AssetUrl>,
    /// The non-PBR specular exponent of this material. If specified alongside `specular`, it will be translated to a PBR equivalent.
    pub specular_exponent: Option<f32>,

    /// The sampler used by every texture in this material. Defaults to a sampler with `Linear` min/mag/mip filter modes and `ClampToEdge` wrap modes across uvw-coordinates.
    pub sampler: Option<SamplerKey>,
}
impl PipelinePbrMaterial {
    pub async fn to_mat(
        &self,
        ctx: &PipelineCtx,
        source_root: &AbsAssetUrl,
        out_root: &AbsAssetUrl,
    ) -> anyhow::Result<PbrMaterialDesc> {
        let pipe_image =
            |path: &Option<AssetUrl>| -> BoxFuture<'_, anyhow::Result<Option<AssetUrl>>> {
                let source_root = source_root.clone();
                let path = path.clone();
                let ctx = ctx.clone();
                async move {
                    if let Some(path) = path {
                        Ok(Some(AssetUrl::from(
                            PipeImage::resolve(&ctx, path.resolve(&source_root).unwrap())
                                .get(ctx.assets())
                                .await?,
                        )))
                    } else {
                        Ok(None)
                    }
                }
                .boxed()
            };
        Ok(PbrMaterialDesc {
            name: self.name.clone(),
            source: self.source.clone(),
            base_color: pipe_image(&self.base_color).await?,
            opacity: pipe_image(&self.opacity).await?,
            normalmap: pipe_image(&self.normalmap).await?,
            metallic_roughness: if let Some(url) = &self.metallic_roughness {
                Some(
                    PipeImage::resolve(ctx, url.resolve(source_root).unwrap())
                        .get(ctx.assets())
                        .await?
                        .into(),
                )
            } else if let Some(specular) = &self.specular {
                let specular_exponent = self.specular_exponent.unwrap_or(1.);
                Some(
                    PipeImage::resolve(ctx, specular.resolve(source_root).unwrap())
                        .transform("mr_from_s", move |image, _| {
                            for p in image.pixels_mut() {
                                let specular =
                                    1. - (1. - p[1] as f32 / 255.).powf(specular_exponent);
                                p[0] = (specular * 255.) as u8;
                                p[1] = ((1. - specular) * 255.) as u8;
                                p[2] = 0;
                                p[3] = 255;
                            }
                        })
                        .get(ctx.assets())
                        .await?
                        .into(),
                )
            } else {
                None
            },

            base_color_factor: self.base_color_factor,
            emissive_factor: self.emissive_factor,
            transparent: self.transparent,
            alpha_cutoff: self.alpha_cutoff,
            double_sided: self.double_sided,
            metallic: self.metallic.unwrap_or(1.),
            roughness: self.roughness.unwrap_or(1.),
            sampler: self.sampler,
        }
        .relative_path_from(out_root))
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
    pub fn new_boxed(name: &'static str, func: F) -> Box<dyn ImageTransformer> {
        Box::new(Self {
            func: Arc::new(func),
            name,
        })
    }
}
impl<F: Fn(&mut RgbaImage, Option<&RgbaImage>) + Sync + Send + 'static> Clone
    for FnImageTransformer<F>
{
    fn clone(&self) -> Self {
        Self {
            func: self.func.clone(),
            name: self.name,
        }
    }
}
impl<F: Fn(&mut RgbaImage, Option<&RgbaImage>) + Sync + Send + 'static> std::fmt::Debug
    for FnImageTransformer<F>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FnImageTransformer")
            .field("name", &self.name)
            .finish()
    }
}
impl<F: Fn(&mut RgbaImage, Option<&RgbaImage>) + Sync + Send + 'static> ImageTransformer
    for FnImageTransformer<F>
{
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
    pub fn resolve(ctx: &PipelineCtx, source: AbsAssetUrl) -> Self {
        Self::new(ctx.get_downloadable_url(&source).unwrap().clone())
    }
    pub fn new(source: AbsAssetUrl) -> Self {
        PipeImage {
            source,
            second_source: None,
            transform: None,
            cap_texture_sizes: None,
        }
    }
    pub fn transform<F: Fn(&mut RgbaImage, Option<&RgbaImage>) + Sync + Send + 'static>(
        mut self,
        transform_name: &'static str,
        transform: F,
    ) -> Self {
        self.transform = Some(FnImageTransformer::new_boxed(transform_name, transform));
        self
    }
    pub fn cap_texture_size(mut self, cap_texture_sizes: Option<ModelTextureSize>) -> Self {
        self.cap_texture_sizes = cap_texture_sizes;
        self
    }
}
#[async_trait]
impl AsyncAssetKey<AssetResult<Arc<AbsAssetUrl>>> for PipeImage {
    async fn load(self, assets: AssetCache) -> AssetResult<Arc<AbsAssetUrl>> {
        let ctx = ProcessCtxKey.get(&assets);
        let mut image = (*ImageFromUrl {
            url: self.source.clone(),
        }
        .get(&assets)
        .await
        .with_context(|| format!("Failed to download image {}", self.source))?)
        .clone();
        let mut extension = "png".to_string();
        let second_image = if let Some(second_source) = &self.second_source {
            Some(
                ImageFromUrl {
                    url: second_source.clone(),
                }
                .get(&assets)
                .await
                .with_context(|| format!("Failed to download second image {}", self.source))?,
            )
        } else {
            None
        };
        let path = ctx.in_root.relative_path(self.source.decoded_path());
        let mut data = Cursor::new(Vec::new());
        tokio::task::block_in_place(|| {
            if let Some(transform) = &self.transform {
                transform.transform(&mut image, second_image.as_deref());
                extension = format!("{}.png", transform.name());
            }
            if let Some(size) = self.cap_texture_sizes {
                cap_texture_size(&mut image, size.size());
            }
            image.write_to(&mut data, ImageOutputFormat::Png).unwrap();
        });
        Ok(Arc::new(
            (ctx.write_file)(
                path.with_extension(extension).to_string(),
                data.into_inner(),
            )
            .await,
        ))
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
        Ok(Arc::new(
            download_image(&assets, &self.url).await?.into_rgba8(),
        ))
    }
}
