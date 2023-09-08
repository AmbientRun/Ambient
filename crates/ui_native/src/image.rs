use std::{borrow::Cow, sync::Arc};

use ambient_core::{asset_cache, gpu, mesh, transform::*, ui_scene};
use ambient_element::{
    use_async, use_memo_with, Element, ElementComponent, ElementComponentExt, Hooks,
};
use ambient_gpu::{
    sampler::SamplerKey,
    std_assets::{DefaultNormalMapViewKey, PixelTextureViewKey},
    texture::TextureView,
    texture_loaders::TextureFromBytes,
};
use ambient_meshes::UIRectMeshKey;
use ambient_native_std::{
    asset_cache::{AsyncAssetKeyExt, SyncAssetKeyExt},
    cb, CowStr,
};
use ambient_renderer::{
    color, gpu_primitives_lod, gpu_primitives_mesh, material,
    materials::pbr_material::{
        get_pbr_shader_unlit, PbrMaterial, PbrMaterialConfig, PbrMaterialParams,
    },
    primitives, renderer_shader, SharedMaterial,
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
        let gpu = hooks.world.resource(gpu()).clone();
        let texture_id = texture.as_ref().map(|x| x.texture.id);
        let mat = use_memo_with(hooks, texture_id, move |_, _| {
            texture.map(|texture| {
                SharedMaterial::new(PbrMaterial::new(
                    &gpu,
                    &assets,
                    PbrMaterialConfig {
                        source: "Image".to_string(),
                        name: "Image".to_string(),
                        params: PbrMaterialParams::default(),
                        base_color: texture,
                        normalmap: DefaultNormalMapViewKey.get(&assets),
                        metallic_roughness: PixelTextureViewKey::white().get(&assets),
                        sampler: SamplerKey::LINEAR_CLAMP_TO_EDGE.get(&assets),
                        transparent: None,
                        double_sided: None,
                        depth_write_enabled: None,
                    },
                ))
            })
        });
        let assets = hooks.world.resource(asset_cache());
        let el = UIBase
            .el()
            .init(width(), 100.)
            .init(height(), 100.)
            .init(mesh(), UIRectMeshKey.get(assets))
            .init_default(mesh_to_local())
            .init_default(mesh_to_local_from_size())
            .init(renderer_shader(), cb(get_pbr_shader_unlit))
            .init(primitives(), vec![])
            .init_default(gpu_primitives_mesh())
            .init_default(gpu_primitives_lod())
            .init(ui_scene(), ())
            .init(color(), Vec4::ONE);

        if let Some(mat) = mat {
            el.with(material(), mat)
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

        let texture = use_async(hooks, |w| {
            let assets = w.resource(asset_cache()).clone();
            async move {
                TextureFromBytes::new(bytes, Some(label))
                    .get(&assets)
                    .await
                    .map(|x| Arc::new(x.create_view(&Default::default())))
            }
        })
        .and_then(Result::ok);

        Image { texture }.el()
    }
}
