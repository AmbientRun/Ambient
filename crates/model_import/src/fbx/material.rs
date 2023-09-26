use std::collections::HashMap;

use ambient_gpu::sampler::SamplerKey;
use ambient_native_std::asset_url::AssetUrl;
use ambient_renderer::materials::pbr_material::PbrMaterialDesc;
use fbxcel::tree::v7400::NodeHandle;
use glam::{vec3, Vec3};

use crate::{
    dotdot_path,
    model_crate::{AssetLoc, ModelCrate},
    TextureResolver,
};

// Valid keys for FbxMaterial::textures:
pub const DIFFUSE_COLOR_KEY: &str = "DiffuseColor";
// pub const TRANSPARENCY_FACTOR_KEY: &str = "TransparencyFactor";
// pub const TRANSPARENT_COLOR_KEY: &str = "TransparentColor";
pub const NORMAL_MAP_KEY: &str = "NormalMap";
// pub const SPECULAR_FACTOR_KEY: &str = "SpecularFactor";
pub const SPECULAR_COLOR_KEY: &str = "SpecularColor";
// pub const SHININESS_EXPONENT_KEY: &str = "ShininessExponent";
// pub const REFLECTION_FACTOR_KEY: &str = "ReflectionFactor";
// pub const EMISSIVE_COLOR_KEY: &str = "EmissiveColor";

#[derive(Debug)]
pub struct FbxMaterial {
    pub id: i64,
    pub name: String,

    pub ambient_color: Option<Vec3>,
    pub diffuse_color: Option<Vec3>,
    pub transparency_factor: Option<f32>,
    pub specular_color: Option<Vec3>,
    pub reflection_factor: Option<f32>,
    pub emissive: Option<Vec3>,
    pub shininess: Option<f32>,
    pub opacity: Option<f32>,
    pub reflectivity: Option<f32>,

    pub textures: HashMap<String, i64>,
}
impl FbxMaterial {
    pub fn from_node(node: NodeHandle) -> Self {
        let id = node.attributes()[0].get_i64().unwrap();
        let name = node.attributes()[1]
            .get_string()
            .unwrap()
            .split('\u{0}')
            .next()
            .unwrap()
            .to_string();
        let props = node
            .children()
            .find(|n| n.name() == "Properties70")
            .unwrap();
        let mut material = Self {
            id,
            name,

            ambient_color: None,
            diffuse_color: None,
            transparency_factor: None,
            specular_color: None,
            reflection_factor: None,
            emissive: None,
            shininess: None,
            opacity: None,
            reflectivity: None,

            textures: Default::default(),
        };
        for prop in props.children() {
            let prop_type = prop.attributes()[0].get_string().unwrap();
            match prop_type {
                "AmbientColor" => {
                    material.ambient_color = Some(vec3(
                        prop.attributes()[4].get_f64().unwrap() as f32,
                        prop.attributes()[5].get_f64().unwrap() as f32,
                        prop.attributes()[6].get_f64().unwrap() as f32,
                    ))
                }
                "DiffuseColor" => {
                    material.diffuse_color = Some(vec3(
                        prop.attributes()[4].get_f64().unwrap() as f32,
                        prop.attributes()[5].get_f64().unwrap() as f32,
                        prop.attributes()[6].get_f64().unwrap() as f32,
                    ))
                }
                "TransparencyFactor" => {
                    material.transparency_factor =
                        Some(prop.attributes()[4].get_f64().unwrap() as f32)
                }
                "SpecularColor" => {
                    material.specular_color = Some(vec3(
                        prop.attributes()[4].get_f64().unwrap() as f32,
                        prop.attributes()[5].get_f64().unwrap() as f32,
                        prop.attributes()[6].get_f64().unwrap() as f32,
                    ))
                }
                "ReflectionFactor" => {
                    material.reflection_factor =
                        Some(prop.attributes()[4].get_f64().unwrap() as f32)
                }
                "Emissive" => {
                    material.emissive = Some(vec3(
                        prop.attributes()[4].get_f64().unwrap() as f32,
                        prop.attributes()[5].get_f64().unwrap() as f32,
                        prop.attributes()[6].get_f64().unwrap() as f32,
                    ))
                }
                "Shininess" => {
                    material.shininess = Some(prop.attributes()[4].get_f64().unwrap() as f32)
                }
                "Opacity" => {
                    material.opacity = Some(prop.attributes()[4].get_f64().unwrap() as f32)
                }
                "Reflectivity" => {
                    material.reflectivity = Some(prop.attributes()[4].get_f64().unwrap() as f32)
                }
                _ => {}
            }
        }
        material
    }
    pub fn to_model_material(
        &self,
        source: String,
        textures: &HashMap<i64, FbxTexture>,
        videos: &HashMap<i64, FbxVideo>,
        images: &HashMap<i64, AssetLoc>,
        asset_crate: &mut ModelCrate,
    ) -> PbrMaterialDesc {
        let find_video_with_filename = |filename: &str, exclude_id: i64| {
            videos
                .iter()
                .find(|v| *v.0 != exclude_id && v.1.filename == filename)
                .map(|v| *v.0)
        };
        let get_map = |id: Option<&i64>| {
            let id = *id?;
            let video_id = textures.get(&id)?.video?;
            if let Some(image) = images.get(&video_id) {
                return Some((image.clone(), id));
            } else if let Some(video) = videos.get(&video_id) {
                // NOTE(Fred): For some "videos", the Content field is missing, but I noticed other videos have the same
                // filename. So I'm guessing we're supposed to resolve videos with the same filename to each other?
                // Seemed to work for one model at least.
                if let Some(other_video) = find_video_with_filename(&video.filename, video_id) {
                    return images.get(&other_video).map(|img| (img.clone(), id));
                }
            }
            None
        };
        let img_to_asset =
            |(image, _id): (AssetLoc, i64)| -> AssetUrl { dotdot_path(image.path).into() };
        PbrMaterialDesc {
            name: Some(self.name.to_string()),
            source: Some(source),
            base_color_factor: self
                .diffuse_color
                .map(|x| x.extend(self.opacity.unwrap_or(1.))),
            emissive_factor: self.emissive.map(|x| x.extend(0.)),
            base_color: get_map(self.textures.get(DIFFUSE_COLOR_KEY)).map(img_to_asset),

            normalmap: get_map(self.textures.get(NORMAL_MAP_KEY)).map(img_to_asset),

            // TODO: These PBR "conversions" are probably wrong
            metallic_roughness: get_map(self.textures.get(SPECULAR_COLOR_KEY))
                .map(|x| {
                    let mut img = asset_crate.images.content.get(&x.0.id).unwrap().clone();
                    for pixel in img.pixels_mut() {
                        pixel.0[0] = 0;
                        pixel.0[1] = 255 - pixel.0[0];
                        pixel.0[2] = 0;
                        pixel.0[3] = 255;
                    }
                    (
                        asset_crate.images.insert(format!("{}-mr", x.0.id), img),
                        x.1,
                    )
                })
                .map(img_to_asset),
            double_sided: None,
            transparent: Some(false),
            alpha_cutoff: Some(0.5),
            metallic_factor: 1.0,
            opacity: None,
            roughness_factor: self
                .textures
                .get(SPECULAR_COLOR_KEY)
                .map(|_| 1.)
                .unwrap_or(1.0),

            // TODO: Each FBX texture knows its sampler modes, but Ambient's
            // current material model assumes a single sampler for all textures
            // in a material. Revisit once the renderer supports arbitrary
            // texture-sampler pairs.
            sampler: Some(SamplerKey::LINEAR_CLAMP_TO_EDGE),
        }
    }
}

#[derive(Debug)]
pub struct FbxTexture {
    pub id: i64,
    pub _name: String,
    pub video: Option<i64>,
}
impl FbxTexture {
    pub fn from_node(node: NodeHandle) -> Self {
        let id = node.attributes()[0].get_i64().unwrap();
        let name = node.attributes()[1]
            .get_string()
            .unwrap()
            .split('\u{0}')
            .next()
            .unwrap()
            .to_string();
        let _props = node
            .children()
            .find(|n| n.name() == "Properties70")
            .unwrap();
        Self {
            id,
            _name: name,
            video: None,
        }
    }
}

#[derive(Debug)]
pub struct FbxVideo {
    pub id: i64,
    pub _name: String,
    pub filename: String,
    pub content: Option<FbxVideoContent>,
}
impl FbxVideo {
    pub fn from_node(node: NodeHandle) -> Self {
        let id = node.attributes()[0].get_i64().unwrap();
        let name = node.attributes()[1]
            .get_string()
            .unwrap()
            .split('\u{0}')
            .next()
            .unwrap()
            .to_string();
        let content = node.children().find(|n| n.name() == "Content");
        let filename = node.children().find(|n| n.name() == "Filename").unwrap();
        Self {
            id,
            _name: name,
            filename: filename.attributes()[0].get_string().unwrap().to_string(),
            content: content.and_then(|content| {
                Some(FbxVideoContent(
                    content.attributes().get(0)?.get_binary()?.to_vec(),
                ))
            }),
        }
    }
    pub async fn to_image(&self, texture_resolver: TextureResolver) -> Option<image::RgbaImage> {
        if let Some(content) = &self.content {
            let format = if self.filename.to_lowercase().ends_with(".png") {
                image::ImageFormat::Png
            } else if self.filename.to_lowercase().ends_with(".jpg")
                || self.filename.to_lowercase().ends_with(".jpeg")
            {
                image::ImageFormat::Jpeg
            } else if self.filename.to_lowercase().ends_with(".tga") {
                image::ImageFormat::Tga
            } else if self.filename.to_lowercase().ends_with(".bmp") {
                image::ImageFormat::Bmp
            } else if self.filename.to_lowercase().ends_with(".dds") {
                image::ImageFormat::Dds
            } else {
                panic!("Unsupported texture format: {:?}", self.filename)
            };
            Some(
                image::load_from_memory_with_format(&content.0, format)
                    .unwrap()
                    .to_rgba8(),
            )
        } else {
            texture_resolver(self.filename.clone()).await
        }
    }
}
pub struct FbxVideoContent(Vec<u8>);
impl std::fmt::Debug for FbxVideoContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FbxVideoContent")
            .field(&self.0.len())
            .finish()
    }
}
