use std::ops::{Add, Div, Mul, Sub};

use glam::{Vec2, Vec3, Vec4};
use serde::{Deserialize, Serialize};

pub trait Saturate {
    // Clamps a value between 0 and 1
    fn saturate(&self) -> Self;
}
impl Saturate for f32 {
    fn saturate(&self) -> Self {
        self.clamp(0., 1.)
    }
}
impl Saturate for Vec2 {
    fn saturate(&self) -> Self {
        self.clamp(Vec2::ZERO, Vec2::ONE)
    }
}
impl Saturate for Vec3 {
    fn saturate(&self) -> Self {
        self.clamp(Vec3::ZERO, Vec3::ONE)
    }
}
impl Saturate for Vec4 {
    fn saturate(&self) -> Self {
        self.clamp(Vec4::ZERO, Vec4::ONE)
    }
}

pub fn mix<X: Clone + Copy, Y: Clone + Copy>(a: Y, b: Y, p: X) -> Y
where
    Y: Sub<Y, Output = Y>,
    Y: Mul<X, Output = Y>,
    Y: Add<Y, Output = Y>,
{
    a + (b - a) * p
}

pub fn interpolate<X: Clone + Copy, Y: Clone + Copy>(x: X, x0: X, x1: X, y0: Y, y1: Y) -> Y
where
    X: Sub<X, Output = X>,
    X: Div<X, Output = X>,
    Y: Sub<Y, Output = Y>,
    Y: Add<Y, Output = Y>,
    X: Mul<Y, Output = Y>,
{
    let p = (x - x0) / (x1 - x0);
    y0 + p * (y1 - y0)
}
pub fn interpolate_clamped<X: Clone + Copy, Y: Clone + Copy>(x: X, x0: X, x1: X, y0: Y, y1: Y) -> Y
where
    X: Sub<X, Output = X>,
    X: Div<X, Output = X>,
    Y: Sub<Y, Output = Y>,
    Y: Add<Y, Output = Y>,
    X: Mul<Y, Output = Y>,
    X: Saturate,
{
    let p = (x - x0) / (x1 - x0);
    y0 + p.saturate() * (y1 - y0)
}

#[test]
fn test_interpolate() {
    assert_eq!(interpolate(-1., 0., 1., 0., 1.), -1.);
}

pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let x = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    x * x * (3. - 2. * x)
}

pub fn angle_shortest_dist(a0: f32, a1: f32) -> f32 {
    let max = std::f32::consts::PI * 2.;
    let da = (a1 - a0) % max;
    ((2. * da) % max) - da
}

pub fn angle_lerp(a0: f32, a1: f32, t: f32) -> f32 {
    a0 + angle_shortest_dist(a0, a1) * t
}

pub fn angle_to_position(self_position: glam::Vec2, self_forward: glam::Vec2, other_position: glam::Vec2) -> f32 {
    let delta = other_position - self_position;
    if delta.length() == 0.0 {
        0.0
    } else {
        self_forward.angle_between(delta)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SphericalCoords {
    pub theta: f32,
    pub phi: f32,
    pub radius: f32,
}
impl SphericalCoords {
    pub fn new(theta: f32, phi: f32, radius: f32) -> Self {
        Self { theta, phi, radius }
    }
}
impl Default for SphericalCoords {
    fn default() -> Self {
        Self::new(0., 0., 1.)
    }
}
impl From<SphericalCoords> for glam::Vec3 {
    fn from(coords: SphericalCoords) -> glam::Vec3 {
        glam::vec3(
            coords.theta.sin() * coords.phi.cos() * coords.radius,
            coords.theta.sin() * coords.phi.sin() * coords.radius,
            coords.theta.cos() * coords.radius,
        )
    }
}

pub fn cdf_sample<T>(vector: &[T], weight: fn(usize, &T) -> f32) -> usize {
    let total_weight = vector.iter().enumerate().fold(0., |p, (i, x)| p + weight(i, x));
    let mut culm = 0.;
    let x = rand::random::<f32>();
    for (i, v) in vector.iter().enumerate() {
        culm += weight(i, v) / total_weight;
        if x <= culm {
            return i;
        }
    }
    unreachable!();
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Line(pub Vec3, pub Vec3);

pub trait Round100 {
    fn round100(&self) -> Self;
}
impl Round100 for f32 {
    fn round100(&self) -> f32 {
        (*self * 100.).round() / 100.
    }
}
impl Round100 for Vec2 {
    fn round100(&self) -> Vec2 {
        (*self * 100.).round() / 100.
    }
}
impl Round100 for Vec3 {
    fn round100(&self) -> Vec3 {
        (*self * 100.).round() / 100.
    }
}
impl Round100 for Vec4 {
    fn round100(&self) -> Vec4 {
        (*self * 100.).round() / 100.
    }
}
