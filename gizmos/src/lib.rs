use elements_ecs::components;
use glam::{Mat4, Vec2};

pub mod render;
mod traits;
use dashmap::{mapref::one::RefMut, DashMap};
use elements_std::math::Line;
use glam::Vec3;
pub use traits::*;

components!("gizmos", {
    gizmos: Gizmos,
});

#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum GizmoPrimitive {
    Sphere { origin: Vec3, radius: f32, color: Vec3, border_width: f32 },
    Line { start: Vec3, end: Vec3, radius: f32, color: Vec3 },
    Rect { origin: Vec3, extents: Vec2, corner: f32, inner_corner: f32, normal: Vec3, thickness: f32, color: Vec3 },
}
impl From<Line> for GizmoPrimitive {
    fn from(line: Line) -> Self {
        GizmoPrimitive::line(line.0, line.1, 0.2)
    }
}

pub const DEFAULT_WIDTH: f32 = 0.02;
pub const DEFAULT_RADIUS: f32 = 0.2;

impl GizmoPrimitive {
    pub fn sphere(origin: Vec3, radius: f32) -> Self {
        Self::Sphere { origin, radius, color: Vec3::ONE, border_width: radius }
    }

    pub fn torus(origin: Vec3, radius: f32, width: f32) -> Self {
        Self::Sphere { origin, radius, color: Vec3::ONE, border_width: width }
    }

    pub fn rect(origin: Vec3, extents: Vec2, corner_radius: f32, normal: Vec3) -> Self {
        Self::Rect { origin, extents, thickness: extents.max_element(), color: Vec3::ONE, corner: corner_radius, normal, inner_corner: 0. }
    }

    pub fn wire_rect(origin: Vec3, extents: Vec2, corner_radius: f32, inner_corner_radius: f32, thickness: f32, normal: Vec3) -> Self {
        Self::Rect { origin, thickness, color: Vec3::ONE, corner: corner_radius, inner_corner: inner_corner_radius, normal, extents }
    }

    pub fn line(start: Vec3, end: Vec3, radius: f32) -> Self {
        Self::Line { start, end, radius, color: Vec3::ONE }
    }

    pub fn ray(origin: Vec3, dir: Vec3, radius: f32) -> Self {
        Self::Line { start: origin, end: origin + dir, radius, color: Vec3::ONE }
    }

    pub fn transform(self, t: Mat4) -> Self {
        let scale = t.transform_vector3(Vec3::X).length();
        match self {
            Self::Sphere { origin, radius, color, border_width } => {
                Self::Sphere { origin: t.transform_point3(origin), radius: radius * scale, border_width: border_width * scale, color }
            }
            Self::Line { start, end, radius, color } => Self::Line {
                start: t.transform_point3(start),
                end: t.transform_point3(end),
                radius: t.transform_vector3(Vec3::X * radius).length(),
                color,
            },
            Self::Rect { origin, extents, corner, inner_corner, normal, thickness, color } => Self::Rect {
                origin: t.transform_point3(origin),
                extents: extents * scale,
                corner,
                inner_corner,
                normal: t.transform_vector3(normal).normalize(),
                thickness: thickness * scale,
                color,
            },
        }
    }

    pub fn with_color(self, color: Vec3) -> Self {
        match self {
            Self::Sphere { origin, radius, color: _, border_width } => Self::Sphere { origin, radius, color, border_width },
            Self::Line { start, end, radius, color: _ } => Self::Line { start, end, radius, color },
            Self::Rect { origin, extents, corner, inner_corner, normal, thickness, color: _ } => {
                Self::Rect { origin, extents, corner, inner_corner, normal, thickness, color }
            }
        }
    }

    pub fn with_size(self, size: f32) -> Self {
        match self {
            Self::Sphere { origin, radius: _, color, border_width } => Self::Sphere { origin, radius: size, color, border_width },
            Self::Line { start, end, radius: _, color } => Self::Line { start, end, radius: size, color },
            Self::Rect { origin, extents, corner, inner_corner, normal, thickness, color } => {
                Self::Rect { origin, extents: extents.normalize_or_zero() * size, corner, inner_corner, normal, thickness, color }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cuboid {
    origin: Vec3,
    extents: Vec3,
    color: Vec3,
    thickness: f32,
}

impl Cuboid {
    pub fn new(origin: Vec3, extents: Vec3, color: Vec3, thickness: f32) -> Self {
        Self { origin, extents, color, thickness }
    }
}

#[derive(Debug, Clone)]
pub struct Gizmos {
    scopes: DashMap<u64, GizmoScope>,
}

impl Gizmos {
    pub fn new() -> Self {
        Self { scopes: DashMap::new() }
    }

    pub fn scope(&self, scope: u64) -> RefMut<u64, GizmoScope> {
        let scope = self.scopes.entry(scope).and_modify(|s| s.clear()).or_default();
        scope
    }

    pub fn scopes(&self) -> dashmap::iter::Iter<u64, GizmoScope> {
        self.scopes.iter()
    }

    pub fn with_scope(&self, scope: u64, f: impl FnOnce(&mut GizmoScope)) -> &Self {
        let mut scope = self.scope(scope);
        f(&mut scope);
        self
    }
}

impl Default for Gizmos {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct GizmoScope {
    primitives: Vec<GizmoPrimitive>,
}

impl GizmoScope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn draw(&mut self, gizmo: impl Gizmo) -> &mut Self {
        self.primitives.extend(gizmo.into_gizmo_primitives());
        self
    }

    fn clear(&mut self) {
        self.primitives.clear()
    }
}
