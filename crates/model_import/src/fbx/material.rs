use std::collections::HashMap;

use elements_renderer::materials::pbr_material::PbrMaterialFromUrl;
use elements_std::asset_url::{AbsAssetUrlOrRelativePath, TypedAssetUrl};
use fbxcel::tree::v7400::NodeHandle;
use glam::{vec3, Vec3};

use crate::{
    dotdot_path, model_crate::{AssetLoc, ModelCrate}
};

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

    pub diffuse_color_texture: Option<i64>,
    pub alpha_texture: Option<i64>,
    pub normalmap: Option<i64>,
    pub specular_factor_texture: Option<i64>,
    pub specular_color_texture: Option<i64>,
    pub shininess_exponent_texture: Option<i64>,
    pub reflection_factor_texture: Option<i64>,
}
impl FbxMaterial {
    pub fn from_node(node: NodeHandle) -> Self {
        let id = node.attributes()[0].get_i64().unwrap();
        let name = node.attributes()[1].get_string().unwrap().split('\u{0}').next().unwrap().to_string();
        let props = node.children().find(|n| n.name() == "Properties70").unwrap();
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

            diffuse_color_texture: None,
            alpha_texture: None,
            normalmap: None,
            specular_factor_texture: None,
            specular_color_texture: None,
            shininess_exponent_texture: None,
            reflection_factor_texture: None,
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
                "TransparencyFactor" => material.transparency_factor = Some(prop.attributes()[4].get_f64().unwrap() as f32),
                "SpecularColor" => {
                    material.specular_color = Some(vec3(
                        prop.attributes()[4].get_f64().unwrap() as f32,
                        prop.attributes()[5].get_f64().unwrap() as f32,
                        prop.attributes()[6].get_f64().unwrap() as f32,
                    ))
                }
                "ReflectionFactor" => material.reflection_factor = Some(prop.attributes()[4].get_f64().unwrap() as f32),
                "Emissive" => {
                    material.emissive = Some(vec3(
                        prop.attributes()[4].get_f64().unwrap() as f32,
                        prop.attributes()[5].get_f64().unwrap() as f32,
                        prop.attributes()[6].get_f64().unwrap() as f32,
                    ))
                }
                "Shininess" => material.shininess = Some(prop.attributes()[4].get_f64().unwrap() as f32),
                "Opacity" => material.opacity = Some(prop.attributes()[4].get_f64().unwrap() as f32),
                "Reflectivity" => material.reflectivity = Some(prop.attributes()[4].get_f64().unwrap() as f32),
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
    ) -> PbrMaterialFromUrl {
        let find_video_with_filename =
            |filename: &str, exclude_id: i64| videos.iter().find(|v| *v.0 != exclude_id && v.1.filename == filename).map(|v| *v.0);
        let get_map = |id: Option<i64>| {
            let id = id?;
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
        let img_to_asset = |(image, _id): (AssetLoc, i64)| -> AbsAssetUrlOrRelativePath { dotdot_path(&image.path).into() };
        PbrMaterialFromUrl {
            name: Some(self.name.to_string()),
            source: Some(source),
            base_color_factor: self.diffuse_color.map(|x| x.extend(self.opacity.unwrap_or(1.))),
            emissive_factor: self.emissive.map(|x| x.extend(0.)),
            base_color: get_map(self.diffuse_color_texture).map(img_to_asset),

            normalmap: get_map(self.normalmap).map(img_to_asset),

            // TODO: These PBR "conversions" are probably wrong
            metallic_roughness: get_map(self.specular_color_texture)
                .map(|x| {
                    let mut img = asset_crate.images.content.get(&x.0.id).unwrap().clone();
                    for pixel in img.pixels_mut() {
                        pixel.0[0] = 0;
                        pixel.0[1] = 255 - pixel.0[0];
                        pixel.0[2] = 0;
                        pixel.0[3] = 255;
                    }
                    (asset_crate.images.insert(format!("{}-mr", x.0.id), img), x.1)
                })
                .map(img_to_asset),
            double_sided: None,
            transparent: Some(false),
            alpha_cutoff: Some(0.5),
            metallic: 0.0,
            opacity: None,
            roughness: self.specular_color_texture.map(|_| 1.).unwrap_or(0.8),
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
        let name = node.attributes()[1].get_string().unwrap().split('\u{0}').next().unwrap().to_string();
        let _props = node.children().find(|n| n.name() == "Properties70").unwrap();
        Self { id, _name: name, video: None }
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
        let name = node.attributes()[1].get_string().unwrap().split('\u{0}').next().unwrap().to_string();
        let content = node.children().find(|n| n.name() == "Content");
        let filename = node.children().find(|n| n.name() == "Filename").unwrap();
        Self {
            id,
            _name: name,
            filename: filename.attributes()[0].get_string().unwrap().to_string(),
            content: content.and_then(|content| Some(FbxVideoContent(content.attributes().get(0)?.get_binary()?.to_vec()))),
        }
    }
    pub fn to_image(&self) -> Option<image::RgbaImage> {
        if let Some(content) = &self.content {
            let format = if self.filename.to_lowercase().ends_with(".png") {
                image::ImageFormat::Png
            } else if self.filename.to_lowercase().ends_with(".jpg") || self.filename.to_lowercase().ends_with(".jpeg") {
                image::ImageFormat::Jpeg
            } else if self.filename.to_lowercase().ends_with(".tga") {
                image::ImageFormat::Tga
            } else if self.filename.to_lowercase().ends_with(".bmp") {
                image::ImageFormat::Bmp
            } else {
                todo!("Unsupported texture format: {:?}", self.filename)
            };
            Some(image::load_from_memory_with_format(&content.0, format).unwrap().to_rgba8())
        } else {
            None
        }
    }
}
pub struct FbxVideoContent(Vec<u8>);
impl std::fmt::Debug for FbxVideoContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FbxVideoContent").field(&self.0.len()).finish()
    }
}
