use std::iter::{once, FusedIterator, Once};

use glam::{Vec3, Vec3Swizzles};

use super::{Cuboid, GizmoPrimitive};

/// Defines any kind of composed gizmo.
/// Is automatically defined for all types that can be converted into a
/// Primitive iterator
pub trait Gizmo {
    type Items: IntoIterator<Item = GizmoPrimitive>;
    fn into_gizmo_primitives(self) -> Self::Items;
}

impl<I> Gizmo for I
where
    I: IntoIterator<Item = GizmoPrimitive>,
{
    type Items = I;

    fn into_gizmo_primitives(self) -> Self::Items {
        self
    }
}

pub trait GizmoExt: Sized {
    type Color;
    type Size;
    // Hint the gizmo color, if applicable
    fn with_color(self, color: Vec3) -> Self::Color;
    // Hint the gizmo size, if applicable
    fn with_size(self, size: f32) -> Self::Size;
}

impl Gizmo for GizmoPrimitive {
    type Items = Once<Self>;

    fn into_gizmo_primitives(self) -> Self::Items {
        once(self)
    }
}

pub struct OptionIter<I> {
    iter: Option<I>,
}

impl<I> Iterator for OptionIter<I>
where
    I: Iterator<Item = GizmoPrimitive>,
{
    type Item = GizmoPrimitive;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.as_mut()?.next()
    }
}

impl<I> FusedIterator for OptionIter<I>
where
    I: FusedIterator + Sized,
    I: Iterator<Item = GizmoPrimitive>,
{
}

// impl<T> Gizmo for Option<T>
// where
//     T: Gizmo,
//     <T::Items as IntoIterator>::IntoIter: FusedIterator,
// {
//     type Items = OptionIter<<T::Items as IntoIterator>::IntoIter>;

//     fn into_primitives(self) -> Self::Items {
//         OptionIter { iter: (self.map(|v| v.into_primitives().into_iter())) }
//     }
// }

// impl Primitive {
//     fn with_color(self, color: Vec3) -> Self {
//         match self {
//             Primitive::Sphere { origin, radius, color: _, border_width } => Self::Sphere { origin, radius, color, border_width },
//             Primitive::Line { start, end, radius, color: _ } => Self::Line { start, end, radius, color },
//             Primitive::Rect { origin, extents, corner, inner_corner, normal, thickness, color: _ } => {
//                 Self::Rect { origin, extents, corner, inner_corner, normal, thickness, color }
//             }
//         }
//     }

//     fn with_size(self, size: f32) -> Self {
//         match self {
//             Primitive::Sphere { origin, radius: _, color, border_width } => Self::Sphere { origin, radius: size, color, border_width },
//             Primitive::Line { start, end, radius: _, color } => Self::Line { start, end, radius: size, color },
//             Primitive::Rect { origin, extents, corner, inner_corner, normal, thickness, color } => {
//                 Self::Rect { origin, extents: extents.normalize_or_zero() * size, corner, inner_corner, normal, thickness, color }
//             }
//         }
//     }
// }

impl<I> GizmoExt for I
where
    I: IntoIterator<Item = GizmoPrimitive>,
{
    type Color = WithColor<I::IntoIter>;

    type Size = WithSize<I::IntoIter>;

    fn with_color(self, color: Vec3) -> Self::Color {
        WithColor {
            p: self.into_iter(),
            color,
        }
    }

    fn with_size(self, size: f32) -> Self::Size {
        WithSize {
            p: self.into_iter(),
            size,
        }
    }
}

// impl Gizmo for Cuboid {
//     type Items = [Primitive; 6];

//     fn into_primitives(self) -> Self::Items {
//         let Self { extents, origin, color, thickness } = self;
//         [
//             Primitive::wire_rect(origin + Vec3::X * extents.x, extents.zy(), 0., 0.1, thickness, Vec3::X, color), // +X
//             Primitive::wire_rect(origin - Vec3::X * extents.x, extents.zy(), 0., 0.1, thickness, -Vec3::X, color), // -X
//             Primitive::wire_rect(origin + Vec3::Y * extents.y, extents.xz(), 0., 0.1, thickness, Vec3::Y, color), // +Y
//             Primitive::wire_rect(origin - Vec3::Y * extents.y, extents.xz(), 0., 0.1, thickness, -Vec3::Y, color), // -Y
//             Primitive::wire_rect(origin + Vec3::Z * extents.z, extents.xy(), 0., 0.1, thickness, Vec3::Z, color), // +Z
//             Primitive::wire_rect(origin - Vec3::Z * extents.z, extents.xy(), 0., 0.1, thickness, -Vec3::Z, color), // -Z
//         ]
//     }
// }

impl IntoIterator for Cuboid {
    type Item = GizmoPrimitive;

    type IntoIter = std::array::IntoIter<GizmoPrimitive, 6>;

    fn into_iter(self) -> Self::IntoIter {
        let Self {
            extents,
            origin,
            color,
            thickness,
        } = self;
        [
            GizmoPrimitive::wire_rect(
                origin + Vec3::X * extents.x,
                extents.zy(),
                0.,
                0.1,
                thickness,
                Vec3::X,
            )
            .with_color(color),
            GizmoPrimitive::wire_rect(
                origin - Vec3::X * extents.x,
                extents.zy(),
                0.,
                0.1,
                thickness,
                -Vec3::X,
            )
            .with_color(color),
            GizmoPrimitive::wire_rect(
                origin + Vec3::Y * extents.y,
                extents.xz(),
                0.,
                0.1,
                thickness,
                Vec3::Y,
            )
            .with_color(color),
            GizmoPrimitive::wire_rect(
                origin - Vec3::Y * extents.y,
                extents.xz(),
                0.,
                0.1,
                thickness,
                -Vec3::Y,
            )
            .with_color(color),
            GizmoPrimitive::wire_rect(
                origin + Vec3::Z * extents.z,
                extents.xy(),
                0.,
                0.1,
                thickness,
                Vec3::Z,
            )
            .with_color(color),
            GizmoPrimitive::wire_rect(
                origin - Vec3::Z * extents.z,
                extents.xy(),
                0.,
                0.1,
                thickness,
                -Vec3::Z,
            )
            .with_color(color),
        ]
        .into_iter()
    }
}

// impl GizmoExt for Cuboid {
//     fn with_color(self, color: Vec3) -> Self {
//         Self { color, ..self }
//     }

//     fn with_size(self, size: f32) -> Self {
//         Self { extents: self.extents.normalize_or_zero() * size, ..self }
//     }

//     type Color = Self;

//     type Size = Self;
// }

// impl Gizmo for Vec3 {
//     type Items = [Primitive; 1];

//     fn into_primitives(self) -> Self::Items {
//         [Primitive::torus(self, DEFAULT_RADIUS, 0.5, vec3(0., 0., 1.))]
//     }
// }

// impl GizmoExt for Vec3 {
//     type Color = WithColor<<<Vec3 as Gizmo>::Items as IntoIterator>::IntoIter>;
//     type Size = WithSize<<<Vec3 as Gizmo>::Items as IntoIterator>::IntoIter>;

//     fn with_color(self, color: Vec3) -> Self::Color {
//         WithColor { color, p: self.into_primitives().into_iter() }
//     }

//     fn with_size(self, size: f32) -> Self::Size {
//         WithSize { size, p: self.into_primitives().into_iter() }
//     }
// }

// impl Gizmo for Ray {
//     type Items = [Primitive; 2];

//     fn into_primitives(self) -> Self::Items {
//         [
//             Primitive::sphere(self.origin, DEFAULT_RADIUS).with_color(vec3(1., 0., 0.)),
//             Primitive::ray(self.origin, self.dir, DEFAULT_WIDTH).with_color(vec3(0.5, 0., 0.)),
//         ]
//     }
// }

pub struct WithColor<I> {
    p: I,
    color: Vec3,
}

// impl<I> Gizmo for WithColor<I>
// where
//     I: std::iter::Iterator<Item = Primitive>,
// {
//     type Items = WithColor<I>;

//     fn into_primitives(self) -> Self {
//         self
//     }
// }

// impl<I> GizmoExt for WithColor<I> {
//     type Color = Self;

//     type Size = WithSize<Self>;

//     fn with_color(self, color: Vec3) -> Self::Color {
//         Self { p: self.p, color }
//     }

//     fn with_size(self, size: f32) -> Self::Size {
//         WithSize { p: self, size }
//     }
// }

impl<I: Iterator<Item = GizmoPrimitive>> Iterator for WithColor<I> {
    type Item = GizmoPrimitive;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.p.next()?.with_color(self.color))
    }
}
pub struct WithSize<I> {
    p: I,
    size: f32,
}

// impl<I> Gizmo for WithSize<I>
// where
//     I: std::iter::Iterator<Item = Primitive>,
// {
//     type Items = WithSize<I>;

//     fn into_primitives(self) -> Self {
//         self
//     }
// }

impl<I: Iterator<Item = GizmoPrimitive>> Iterator for WithSize<I> {
    type Item = GizmoPrimitive;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.p.next()?.with_size(self.size))
    }
}

// impl<I> GizmoExt for WithSize<I> {
//     type Color = WithColor<Self>;

//     type Size = Self;

//     fn with_color(self, color: Vec3) -> Self::Color {
//         WithColor { p: self, color }
//     }

//     fn with_size(self, size: f32) -> Self::Size {
//         WithSize { p: self.p, size }
//     }
// }
