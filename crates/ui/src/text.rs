use std::{num::NonZeroU32, ops::Deref, sync::Arc};

use anyhow::Context;
use async_trait::async_trait;
use elements_core::{asset_cache, async_ecs::async_run, gpu, mesh, name, runtime, transform::*, ui_scene, window_scale_factor};
use elements_ecs::{components, query, query_mut, EntityData, SystemGroup, World};
use elements_element::{element_component, Element, ElementComponentExt, Hooks};
use elements_gpu::{mesh_buffer::GpuMesh, texture::Texture};
use elements_renderer::{color, gpu_primitives, material, primitives, renderer_shader, SharedMaterial};
use elements_std::{
    asset_cache::{AssetCache, AsyncAssetKey, AsyncAssetKeyExt}, asset_url::AbsAssetUrl, download_asset::{AssetResult, BytesFromUrl}, mesh::*, shapes::AABB
};
use glam::*;
use glyph_brush::{
    ab_glyph::{Font, FontArc, PxScale, Rect}, BrushAction, BrushError, GlyphBrush, GlyphBrushBuilder, Section
};
use log::info;
use parking_lot::Mutex;

use crate::{
    layout::*, text_material::{get_text_shader, TextMaterial}, UIBase, UIElement
};

components!("ui", {
    text: String,
    text_case: TextCase,
    font_size: f32,
    font_style: FontStyle,
    font_family: FontFamily,
    font_arc: Arc<FontArc>,

    glyph_brush: Arc<Mutex<GlyphBrush<GlyphVertex>>>,
    text_texture: Arc<Texture>,
});

/// A text element. Use the `text`, `font_size`, `font` and `color` components to set its state
#[element_component(without_el)]
pub fn Text(world: &mut World, _hooks: &mut Hooks) -> Element {
    let scale_factor = *world.resource(window_scale_factor()) as f32;

    UIBase
        .el()
        .init(width(), 1.)
        .init(height(), 1.)
        .init(mesh_to_local(), Mat4::from_scale(Vec3::ONE / scale_factor))
        .init(color(), vec4(0.6, 0.6, 0.6, 1.))
        .init(name(), "Text".to_string())
        .init(ui_scene(), ())
        .init_default(font_family())
        .init_default(font_style())
        .init(font_size(), 12.)
        .init(text(), "".to_string())
}
impl Text {
    pub fn el(value: impl Into<String>) -> Element {
        Text.el().set(text(), value.into())
    }
}
impl From<&str> for UIElement {
    fn from(value: &str) -> Self {
        UIElement(Text.el().set(text(), value.to_string()))
    }
}
impl From<String> for UIElement {
    fn from(value: String) -> Self {
        UIElement(Text.el().set(text(), value))
    }
}
impl From<&String> for UIElement {
    fn from(value: &String) -> Self {
        UIElement(Text.el().set(text(), value.to_string()))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TextCase {
    AsTyped,
    Uppercase,
    Lowercase,
}
impl Default for TextCase {
    fn default() -> Self {
        Self::AsTyped
    }
}
impl TextCase {
    pub fn format(&self, text: impl Into<String>) -> String {
        let text: String = text.into();
        match self {
            TextCase::AsTyped => text,
            TextCase::Uppercase => text.to_uppercase(),
            TextCase::Lowercase => text.to_lowercase(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum FontStyle {
    Bold,
    BoldItalic,
    Medium,
    MediumItalic,
    Regular,
    Italic,
    Light,
    LightItalic,
}
impl Default for FontStyle {
    fn default() -> Self {
        Self::Regular
    }
}

#[derive(Debug, Clone)]
pub enum FontFamily {
    Default,
    Custom(AbsAssetUrl),
    FontAwesome { solid: bool },
    SourceSansPro,
}
impl Default for FontFamily {
    fn default() -> Self {
        Self::Default
    }
}
#[derive(Debug, Clone)]
struct FontDef(FontFamily, FontStyle);
#[async_trait]
impl AsyncAssetKey<Arc<FontArc>> for FontDef {
    async fn load(self, assets: AssetCache) -> Arc<FontArc> {
        match self.0 {
            FontFamily::Default => {
                let font: &'static [u8] = match self.1 {
                    FontStyle::Bold => {
                        include_bytes!("../../../assets/fonts/Ubuntu/Ubuntu Bold Nerd Font Complete.ttf")
                    }
                    FontStyle::BoldItalic => include_bytes!("../../../assets/fonts/Ubuntu/Ubuntu Bold Italic Nerd Font Complete.ttf"),
                    FontStyle::Italic => include_bytes!("../../../assets/fonts/Ubuntu/Ubuntu Italic Nerd Font Complete.ttf"),
                    FontStyle::Light => {
                        include_bytes!("../../../assets/fonts/Ubuntu/Ubuntu Light Nerd Font Complete.ttf")
                    }
                    FontStyle::LightItalic => include_bytes!("../../../assets/fonts/Ubuntu/Ubuntu Light Italic Nerd Font Complete.ttf"),
                    FontStyle::Medium => include_bytes!("../../../assets/fonts/Ubuntu/Ubuntu Medium Nerd Font Complete.ttf"),
                    FontStyle::MediumItalic => include_bytes!("../../../assets/fonts/Ubuntu/Ubuntu Medium Italic Nerd Font Complete.ttf"),
                    FontStyle::Regular => {
                        include_bytes!("../../../assets/fonts/Ubuntu/Ubuntu Nerd Font Complete.ttf")
                    }
                };
                Arc::new(FontArc::try_from_slice(font).unwrap())
            }
            FontFamily::FontAwesome { solid } => Arc::new(
                FontArc::try_from_slice(if solid {
                    include_bytes!("../../../assets/fonts/FontAwesome/Font Awesome 6 Free-Solid-900.otf")
                } else {
                    include_bytes!("../../../assets/fonts/FontAwesome/Font Awesome 6 Free-Regular-400.otf")
                })
                .unwrap(),
            ),
            FontFamily::SourceSansPro => {
                let font: &'static [u8] = match self.1 {
                    FontStyle::Bold => {
                        include_bytes!("../../../assets/fonts/Source_Sans_Pro/SourceSansPro-Bold.ttf")
                    }
                    FontStyle::BoldItalic => include_bytes!("../../../assets/fonts/Source_Sans_Pro/SourceSansPro-BoldItalic.ttf"),
                    FontStyle::Italic => {
                        include_bytes!("../../../assets/fonts/Source_Sans_Pro/SourceSansPro-Italic.ttf")
                    }
                    FontStyle::Light => {
                        include_bytes!("../../../assets/fonts/Source_Sans_Pro/SourceSansPro-Light.ttf")
                    }
                    FontStyle::LightItalic => include_bytes!("../../../assets/fonts/Source_Sans_Pro/SourceSansPro-LightItalic.ttf"),
                    FontStyle::Medium => {
                        include_bytes!("../../../assets/fonts/Source_Sans_Pro/SourceSansPro-SemiBold.ttf")
                    }
                    FontStyle::MediumItalic => include_bytes!("../../../assets/fonts/Source_Sans_Pro/SourceSansPro-SemiBoldItalic.ttf"),
                    FontStyle::Regular => {
                        include_bytes!("../../../assets/fonts/Source_Sans_Pro/SourceSansPro-Regular.ttf")
                    }
                };
                Arc::new(FontArc::try_from_slice(font).unwrap())
            }
            FontFamily::Custom(url) => FontFromUrl(url.clone().into()).get(&assets).await.unwrap(),
        }
    }
}

pub fn systems() -> SystemGroup {
    SystemGroup::new(
        "ui/text",
        vec![
            query(()).incl(text()).excl(renderer_shader()).spawned().to_system(|q, world, qs, _| {
                let assets = world.resource(asset_cache()).clone();
                let gpu = world.resource(gpu()).clone();
                for (id, _) in q.collect_cloned(world, qs) {
                    let texture = Arc::new(Texture::new(
                        gpu.clone(),
                        &wgpu::TextureDescriptor {
                            size: wgpu::Extent3d { width: 256, height: 256, depth_or_array_layers: 1 },
                            mip_level_count: 1,
                            sample_count: 1,
                            dimension: wgpu::TextureDimension::D2,
                            format: wgpu::TextureFormat::R8Unorm,
                            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                            label: Some("Text.texture"),
                        },
                    ));
                    let texture_view = Arc::new(texture.create_view(&wgpu::TextureViewDescriptor::default()));
                    world
                        .add_components(
                            id,
                            EntityData::new()
                                .set(text_texture(), texture)
                                .set(renderer_shader(), get_text_shader(&assets))
                                .set(material(), SharedMaterial::new(TextMaterial::new(assets.clone(), texture_view)))
                                .set(primitives(), vec![])
                                .set_default(gpu_primitives()),
                        )
                        .unwrap();
                }
            }),
            query((font_family().changed(), font_style().changed())).to_system(|q, world, qs, _| {
                for (id, (font_family, font_style)) in q.collect_cloned(world, qs) {
                    let async_run = world.resource(async_run()).clone();
                    let assets = world.resource(asset_cache()).clone();
                    world.resource(runtime()).spawn(async move {
                        let font = FontDef(font_family, font_style).get(&assets).await;
                        async_run.run(move |world| {
                            world.add_component(id, font_arc(), font).ok();
                        });
                    });
                }
            }),
            query(font_arc().changed()).to_system(|q, world, qs, _| {
                for (id, font) in q.collect_cloned(world, qs) {
                    let brush = Arc::new(Mutex::new(GlyphBrushBuilder::using_font(font.deref().clone()).build()));
                    world.add_component(id, glyph_brush(), brush).unwrap();
                }
            }),
            query(window_scale_factor().changed()).to_system(|q, world, qs, _| {
                let scale_factor = match q.iter(world, qs).next() {
                    Some((_, wsf)) => *wsf as f32,
                    None => return,
                };

                // Mark all glyph brushes as dirty to ensure they're rebuilt when the scale factor changes.
                for (_, (mesh_to_local, _), _) in query_mut((mesh_to_local(), glyph_brush()), ()).iter(world, None) {
                    *mesh_to_local = Mat4::from_scale(Vec3::ONE / scale_factor);
                }
            }),
            query((glyph_brush().changed(), text().changed(), font_size().changed(), font_arc()))
                .incl(text_texture())
                .optional_changed(text_case())
                .optional_changed(min_width())
                .to_system(|q, world, qs, _| {
                    let scale_factor = *world.resource(window_scale_factor()) as f32;
                    for (id, (glyph_brush, text, font_size, font)) in q.collect_cloned(world, qs) {
                        let assets = world.resource(asset_cache()).clone();
                        let text = world.get(id, text_case()).unwrap_or_default().format(text);
                        let min_width = world.get(id, min_width()).unwrap_or(0.);
                        let min_height = world.get(id, min_height()).unwrap_or(0.);

                        loop {
                            let process_result =
                                {
                                    let mut brush = glyph_brush.lock();
                                    brush.queue(Section::default().add_text(
                                        glyph_brush::Text::new(&text).with_scale(pt_size_to_px_scale(&*font, font_size, scale_factor)),
                                    ));
                                    brush.process_queued(
                                        |rect, tex_data| {
                                            let gpu = world.resource(gpu());

                                            gpu.queue.write_texture(
                                                wgpu::ImageCopyTexture {
                                                    texture: &world.get_ref(id, text_texture()).unwrap().handle,
                                                    mip_level: 0,
                                                    origin: wgpu::Origin3d { x: rect.min[0], y: rect.min[1], z: 0 },
                                                    aspect: wgpu::TextureAspect::All,
                                                },
                                                tex_data,
                                                wgpu::ImageDataLayout {
                                                    offset: 0,
                                                    bytes_per_row: NonZeroU32::new(rect.width()),
                                                    rows_per_image: NonZeroU32::new(rect.height()),
                                                },
                                                wgpu::Extent3d { width: rect.width(), height: rect.height(), depth_or_array_layers: 1 },
                                            );
                                        },
                                        |vertex_data| GlyphVertex {
                                            tex_coords: vertex_data.tex_coords,
                                            pixel_coords: vertex_data.pixel_coords,
                                        },
                                    )
                                };
                            match process_result {
                                Ok(BrushAction::Draw(vertices)) => {
                                    let has_verts = !vertices.is_empty();
                                    let cpu_mesh = mesh_from_glyph_vertices(vertices);
                                    let bounding = if has_verts { cpu_mesh.aabb().unwrap() } else { AABB::new(Vec3::ZERO, Vec3::ZERO) };
                                    world
                                        .add_components(
                                            id,
                                            EntityData::new()
                                                .set(width(), (bounding.max.x / scale_factor).max(min_width))
                                                .set(height(), (bounding.max.y / scale_factor).max(min_height))
                                                .set(mesh(), GpuMesh::from_mesh(assets.clone(), &cpu_mesh)),
                                        )
                                        .unwrap();
                                    break;
                                }
                                Ok(BrushAction::ReDraw) => {
                                    break;
                                }
                                Err(BrushError::TextureTooSmall { suggested }) => {
                                    let size = wgpu::Extent3d { width: suggested.0, height: suggested.1, depth_or_array_layers: 1 };
                                    let gpu = world.resource(gpu()).clone();
                                    let texture = Arc::new(Texture::new(
                                        gpu,
                                        &wgpu::TextureDescriptor {
                                            size,
                                            mip_level_count: 1,
                                            sample_count: 1,
                                            dimension: wgpu::TextureDimension::D2,
                                            format: wgpu::TextureFormat::R8Unorm,
                                            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                                            label: Some("Text.texture"),
                                        },
                                    ));
                                    glyph_brush.lock().resize_texture(suggested.0, suggested.1);
                                    let view = Arc::new(texture.create_view(&wgpu::TextureViewDescriptor::default()));
                                    world
                                        .add_components(
                                            id,
                                            EntityData::new()
                                                .set(material(), SharedMaterial::new(TextMaterial::new(assets.clone(), view.clone())))
                                                .set(text_texture(), texture),
                                        )
                                        .unwrap();
                                }
                            }
                        }
                    }
                }),
        ],
    )
}

// From: https://docs.rs/glyph_brush/latest/glyph_brush/ab_glyph/trait.Font.html#units
fn pt_size_to_px_scale<F: Font>(font: &F, pt_size: f32, screen_scale_factor: f32) -> PxScale {
    let px_per_em = pt_size * screen_scale_factor; // * (96.0 / 72.0); // this part is used in the example but seems to make the scale wrong, hence disabled
    let units_per_em = font.units_per_em().unwrap();
    let height = font.height_unscaled();
    PxScale::from(px_per_em * height / units_per_em)
}

#[derive(Clone)]
pub struct GlyphVertex {
    pub tex_coords: Rect,
    pub pixel_coords: Rect,
}

fn mesh_from_glyph_vertices(vertices: Vec<GlyphVertex>) -> Mesh {
    let mut positions = Vec::new();
    let mut texcoords = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    for vert in vertices.into_iter() {
        let offset = positions.len() as u32;
        positions.push(vec3(vert.pixel_coords.min.x, vert.pixel_coords.min.y, 0.));
        positions.push(vec3(vert.pixel_coords.max.x, vert.pixel_coords.min.y, 0.));
        positions.push(vec3(vert.pixel_coords.min.x, vert.pixel_coords.max.y, 0.));
        positions.push(vec3(vert.pixel_coords.max.x, vert.pixel_coords.max.y, 0.));

        texcoords.push(vec2(vert.tex_coords.min.x, vert.tex_coords.min.y));
        texcoords.push(vec2(vert.tex_coords.max.x, vert.tex_coords.min.y));
        texcoords.push(vec2(vert.tex_coords.min.x, vert.tex_coords.max.y));
        texcoords.push(vec2(vert.tex_coords.max.x, vert.tex_coords.max.y));

        normals.push(vec3(0., 0., 1.));
        normals.push(vec3(0., 0., 1.));
        normals.push(vec3(0., 0., 1.));
        normals.push(vec3(0., 0., 1.));

        indices.push(offset);
        indices.push(offset + 1);
        indices.push(offset + 2);

        indices.push(offset + 1);
        indices.push(offset + 3);
        indices.push(offset + 2);
    }
    Mesh {
        name: "GlyphMesh".to_string(),
        positions: Some(positions),
        texcoords: vec![texcoords],
        normals: Some(normals),
        indices: Some(indices),
        ..Default::default()
    }
}

#[derive(Debug, Clone)]
pub struct FontFromUrl(AbsAssetUrl);

#[async_trait]
impl AsyncAssetKey<AssetResult<Arc<FontArc>>> for FontFromUrl {
    async fn load(self, assets: elements_std::asset_cache::AssetCache) -> AssetResult<Arc<FontArc>> {
        info!("Downloading font: {}", self.0);
        let data = BytesFromUrl::new(self.0, true).get(&assets).await?;
        let brush = FontArc::try_from_vec(data.deref().clone()).context("Failed to parse font")?;
        Ok(Arc::new(brush))
    }
}
