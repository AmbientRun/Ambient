use std::iter::once;

use glam::{vec2, vec3, Quat, Vec2, Vec3, Vec4};
use itertools::Itertools;
use kiwi_core::{
    asset_cache,
    transform::{mesh_to_local, rotation, translation},
    ui_scene,
};
use kiwi_element::{Element, ElementComponent, ElementComponentExt};
use kiwi_gpu::{self, mesh_buffer::MeshBufferKey};
use kiwi_renderer::{
    color, flat_material::get_flat_shader_unlit, gpu_primitives, material, materials::flat_material::FlatMaterial, primitives,
    renderer_shader, SharedMaterial,
};
use kiwi_std::{asset_cache::SyncAssetKeyExt, cb, mesh::Mesh};

use crate::{height, mesh_to_local_from_size, rect::Rectangle, width, Text, UIBase};

#[derive(Copy, Debug, Clone)]
pub struct GraphStyle {
    pub width: f32,
    pub color: Vec4,
}

impl Default for GraphStyle {
    fn default() -> Self {
        Self { width: 4.0, color: Vec4::X }
    }
}

#[derive(Debug, Clone)]
pub struct Graph {
    pub points: Vec<Vec2>,
    pub guide_style: GraphStyle,
    pub style: GraphStyle,
    pub width: f32,
    pub height: f32,
    pub max_value: f32,
    /// Provide a custom scale for the graph.
    /// If `None` it will be deduced from the points.
    ///
    /// Providing a custom scale can prevent jumping.
    pub x_scale: GraphScaleKind,
    pub y_scale: GraphScaleKind,
    pub x_bounds: Option<(f32, f32)>,
    pub y_bounds: Option<(f32, f32)>,
}

#[derive(Copy, Debug, Clone)]
struct GraphScale {
    screen_size: f32,
    spacing: f32,
    min: f32,
    max: f32,
    offset: f32,

    tick_count: u32,
}

#[derive(Debug, Clone)]
/// Describes how the ticks should be spaced out
pub enum GraphScaleKind {
    Fixed { count: u32 },
    Dynamic { spacing: f32, snap: bool },
}

impl GraphScale {
    fn span(&self) -> f32 {
        (self.max - self.min).abs()
    }

    fn ticks_to_orig(&self) -> Option<u32> {
        if self.min <= 0.0 && self.max >= 0.0 {
            Some((self.min.abs() / self.spacing) as u32)
        } else {
            None
        }
    }

    /// Return the number of pixels separating each tick, including the tick
    /// itself
    fn screen_spacing(&self) -> f32 {
        self.spacing * self.screen_size / self.span()
    }
}

impl Default for GraphScaleKind {
    fn default() -> Self {
        Self::Dynamic { spacing: 32.0, snap: true }
    }
}

impl GraphScaleKind {
    fn to_scale(&self, screen_size: f32, min: f32, max: f32) -> GraphScale {
        let get_spacing =
            |spacing| (-9..).map(move |n| 10.0_f32.powi(n)).flat_map(|m| [m, 2.0 * m, 5.0 * m]).find(|&v| v >= spacing).unwrap();
        let (min, max, spacing) = match *self {
            GraphScaleKind::Fixed { count } => {
                let span = (max - min).abs();

                let spacing = get_spacing(span / count as f32);
                (min, max, spacing)
            }
            GraphScaleKind::Dynamic { spacing, snap } => {
                let span = (max - min).abs();
                let spacing = get_spacing(span * spacing / screen_size);

                let (min, max) = if snap { ((min / spacing).floor() * spacing, (max / spacing).ceil() * spacing) } else { (min, max) };

                (min, max, spacing)
            }
        };

        // let min = (min / spacing).floor() * spacing;
        let offset = min % spacing;

        let span = (max - min).abs();

        let tick_count = (span / spacing) as u32;

        GraphScale { screen_size, spacing, min, max, offset, tick_count }
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self {
            points: Default::default(),
            guide_style: GraphStyle { width: 2.0, color: Vec4::ONE },
            style: Default::default(),
            width: 161.8,
            height: 100.0,
            max_value: 1e9,
            x_scale: Default::default(),
            y_scale: Default::default(),
            x_bounds: None,
            y_bounds: None,
        }
    }
}

impl ElementComponent for Graph {
    fn render(self: Box<Self>, world: &mut kiwi_ecs::World, _hooks: &mut kiwi_element::Hooks) -> kiwi_element::Element {
        let assets = world.resource(asset_cache()).clone();
        let Self { points, guide_style, style, width, height, max_value, x_scale, y_scale, x_bounds, y_bounds } = *self;

        let points = points.into_iter().filter(|v| v.x.is_normal() && v.y.is_normal() && v.x.abs() < max_value && v.y.abs() < max_value);

        let point_count = points.clone().count();

        let mesh_buffer = MeshBufferKey.get(&assets);
        let mut mesh_buffer = mesh_buffer.lock();

        // points.clone().minmax_by_key(|v| v.x)

        let x_bounds = x_bounds.or_else(|| points.clone().map(|v| v.x).minmax().into_option()).unwrap_or((-1.0, 1.0));
        let y_bounds = y_bounds.or_else(|| points.clone().map(|v| v.y).minmax().into_option()).unwrap_or((-1.0, 1.0));

        let left = vec2(x_bounds.0, y_bounds.0);
        let right = vec2(x_bounds.1, y_bounds.1);

        // let (left, right) = points.clone().fold((Vec2::ZERO, Vec2::ZERO), |(left, right), x| (left.min(x), right.max(x)));
        // Make sure the bound is not 0,0
        let right = right.max(left + Vec2::ONE);

        let x_scale = x_scale.to_scale(width, left.x, right.x);
        let y_scale = y_scale.to_scale(height, left.y, right.y);

        let span = vec2(x_scale.span(), y_scale.span());

        let line_width = style.width / (vec2(width, height) * 2.0);

        let size = vec2(width, height);

        let to_screen_space = |v: Vec2| {
            let v = v.clamp(left, right);

            let p = (v - left) / span;
            vec2(p.x, 1.0 - p.y)
        };

        let screen_points = points.clone().map(to_screen_space);

        let normals = once(Vec2::X)
            .chain(screen_points.clone().tuple_windows().map(|(a, b, c)| {
                let rel_a = b - a;
                let rel_c = c - b;

                let inc = (rel_a * rel_a.x + rel_c * rel_c.x) / (rel_a.x + rel_c.x);
                inc.perp().normalize_or_zero()
            }))
            .chain(once(Vec2::X));

        let points = screen_points
            .zip(normals)
            .flat_map(|(p, norm)| {
                let off = norm * line_width;
                [p + off, p - off]
            })
            .map(|v| v.extend(0.0))
            .collect_vec();

        debug_assert_eq!(points.len(), point_count * 2);

        let indices =
            (0..point_count.saturating_sub(1) as u32).map(|i| i * 2).flat_map(|i| [i, 1 + i, 2 + i, 1 + i, 3 + i, 2 + i]).collect_vec();

        let mesh = Mesh { name: "Graph Mesh".to_string(), positions: Some(points), indices: Some(indices), ..Default::default() };

        let mesh = mesh_buffer.insert(&mesh);

        let origin = to_screen_space(Vec2::ZERO) * size;
        let guides = vec![
            Guide { style: guide_style, len: width, scale: x_scale, dir: Vec2::X, align: vec2(0.0, -origin.y), show_0: false }.el(),
            Guide { style: guide_style, len: height, scale: y_scale, dir: -Vec2::Y, align: vec2(0.0, -height), show_0: false }.el(),
        ];

        Element::from(UIBase)
            .init_default(mesh_to_local())
            .init_default(primitives())
            .init_default(gpu_primitives())
            .children(guides)
            .init_default(mesh_to_local_from_size())
            .init(crate::width(), width)
            .init(crate::height(), height)
            .init(crate::scale(), Vec3::ONE)
            .init(renderer_shader(), cb(get_flat_shader_unlit))
            .init(material(), SharedMaterial::new(FlatMaterial::new(assets, style.color, None)))
            .init(color(), Vec4::ONE)
            .init(ui_scene(), ())
            .set(kiwi_core::mesh(), mesh)
    }
}

#[derive(Debug, Clone)]
struct Guide {
    style: GraphStyle,
    scale: GraphScale,
    len: f32,
    dir: Vec2,
    align: Vec2,
    show_0: bool,
}

impl ElementComponent for Guide {
    fn render(self: Box<Self>, _: &mut kiwi_ecs::World, _: &mut kiwi_element::Hooks) -> Element {
        let Self { style, scale, len, dir, align, show_0, .. } = *self;

        let rot = Quat::from_rotation_arc(Vec3::X, dir.extend(0.0));

        let text_rot = rot.inverse();
        let screen_offset = scale.offset / scale.span() * scale.screen_size;
        let ticks = (0..=scale.tick_count)
            .map(|i| {
                let val = (-scale.offset as f64 + scale.min as f64 + i as f64 * scale.spacing as f64) as f32;
                (i, val)
            })
            .filter(|&(i, v)| ((show_0) || Some(i) != scale.ticks_to_orig()) && v >= scale.min)
            .map(|(i, val)| {
                let f = i as f32 * scale.screen_spacing();

                let pos = Vec2::X * (f - screen_offset);

                Tick { style, height: style.width * 4.0, pos, text_rot, val }.el()
            })
            .chain([])
            .collect_vec();

        let pos = vec3(align.x, 1.0 - align.y, 0.0);

        Rectangle
            .el()
            .children(ticks)
            .set(width(), len)
            .set(height(), style.width)
            .set(color(), style.color)
            .set(rotation(), rot)
            .set(translation(), pos)
    }
}

#[derive(Debug, Clone)]
struct Tick {
    style: GraphStyle,
    height: f32,
    pos: Vec2,
    text_rot: Quat,

    val: f32,
}

impl ElementComponent for Tick {
    fn render(self: Box<Self>, _: &mut kiwi_ecs::World, _: &mut kiwi_element::Hooks) -> Element {
        let Self { style, height, pos, text_rot, val } = *self;
        Rectangle
            .el()
            .set(width(), self.style.width)
            .set(super::layout::height(), height)
            .children(vec![Text::el(format!("{val:?}"))
                .set(color(), self.style.color)
                .set(translation(), Vec3::Y * height)
                .set(rotation(), text_rot)])
            .set(color(), style.color)
            .set(translation(), pos.extend(0.0))
    }
}
