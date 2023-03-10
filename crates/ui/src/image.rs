use std::{borrow::Cow, sync::Arc};

use ambient_core::{asset_cache, mesh, transform::*, ui_scene};
use ambient_element::{Element, ElementComponent, ElementComponentExt, Hooks};
use ambient_gpu::{
    std_assets::{DefaultNormalMapViewKey, PixelTextureViewKey},
    texture::TextureView,
    texture_loaders::{TextureFromBytes, TextureFromUrl},
};
use ambient_meshes::UIRectMeshKey;
use ambient_renderer::{
    color, gpu_primitives_lod, gpu_primitives_mesh, material,
    materials::pbr_material::{get_pbr_shader_unlit, PbrMaterial, PbrMaterialConfig, PbrMaterialParams},
    primitives, renderer_shader, SharedMaterial,
};
use ambient_std::{
    asset_cache::{AsyncAssetKeyExt, SyncAssetKeyExt},
    asset_url::AbsAssetUrl,
    cb, CowStr,
};
use glam::*;

use super::UIBase;
use crate::layout::*;

#[derive(Clone, Debug)]
pub struct Image {
    pub texture: Option<Arc<TextureView>>,
}
impl ElementComponent for Image {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let Image { texture } = *self;
        let assets = hooks.world.resource(asset_cache()).clone();
        let texture_id = texture.as_ref().map(|x| x.texture.id);
        let mat = hooks.use_memo_with(texture_id, move |_, _| {
            texture.map(|texture| {
                SharedMaterial::new(PbrMaterial::new(
                    assets.clone(),
                    PbrMaterialConfig {
                        source: "Image".to_string(),
                        name: "Image".to_string(),
                        params: PbrMaterialParams::default(),
                        base_color: texture,
                        normalmap: DefaultNormalMapViewKey.get(&assets),
                        metallic_roughness: PixelTextureViewKey::white().get(&assets),
                        transparent: None,
                        double_sided: None,
                        depth_write_enabled: None,
                    },
                ))
            })
        });
        let assets = hooks.world.resource(asset_cache()).clone();
        let el = UIBase
            .el()
            .init(width(), 100.)
            .init(height(), 100.)
            .init(mesh(), UIRectMeshKey.get(&assets))
            .init_default(mesh_to_local())
            .init_default(mesh_to_local_from_size())
            .init(renderer_shader(), cb(get_pbr_shader_unlit))
            .init(primitives(), vec![])
            .init_default(gpu_primitives_mesh())
            .init_default(gpu_primitives_lod())
            .init(ui_scene(), ())
            .init(color(), Vec4::ONE);

        if let Some(mat) = mat {
            el.set(material(), mat)
        } else {
            el
        }
    }
}

#[derive(Clone, Debug)]
pub struct ImageFromBytes {
    pub bytes: Cow<'static, [u8]>,
    pub label: CowStr,
}

impl ElementComponent for ImageFromBytes {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let Self { bytes, label } = *self;

        let texture =
            hooks
                .use_async(|w| {
                    let assets = w.resource(asset_cache()).clone();
                    async move {
                        TextureFromBytes::new(bytes, Some(label)).get(&assets).await.map(|x| Arc::new(x.create_view(&Default::default())))
                    }
                })
                .and_then(Result::ok);

        Image { texture }.el()
    }
}

#[derive(Clone, Debug)]
pub struct ImageFromUrl {
    pub url: String,
}

impl ElementComponent for ImageFromUrl {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let ImageFromUrl { url } = *self;

        let texture = hooks
            .use_async(|w| {
                let assets = w.resource(asset_cache()).clone();
                async move {
                    TextureFromUrl { url: AbsAssetUrl::parse(url)?, format: wgpu::TextureFormat::Rgba8UnormSrgb }
                        .get(&assets)
                        .await
                        .map(|x| Arc::new(x.create_view(&Default::default())))
                }
            })
            .and_then(Result::ok);

        Image { texture }.el()
    }
}
