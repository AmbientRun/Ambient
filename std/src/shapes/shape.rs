use std::sync::Arc;

use glam::{vec3, Mat4, Vec3};

use super::{Cuboid, Sphere};

pub trait Shape: std::fmt::Debug {
    /// Returns the furthest point in the given direction.
    ///
    /// *Note*: dir must be normalized
    fn support(&self, dir: Vec3) -> Vec3;
    fn support_dist(&self, dir: Vec3) -> f32 {
        self.support(dir).dot(dir)
    }
}

#[derive(Debug, Clone)]
pub struct TransformedShape<T: Shape> {
    shape: T,
    transform: Mat4,
}

impl<T: Shape> TransformedShape<T> {
    pub fn new(shape: T, transform: Mat4) -> Self {
        Self { shape, transform }
    }
}

impl<T: Shape> Shape for TransformedShape<T> {
    fn support(&self, dir: Vec3) -> Vec3 {
        let p = self.shape.support(self.transform.inverse().transform_vector3(dir).normalize());
        self.transform.transform_point3(p)
    }
}

impl Shape for Sphere {
    fn support(&self, dir: Vec3) -> Vec3 {
        self.radius * dir
    }
}

impl Shape for Cuboid {
    fn support(&self, dir: Vec3) -> Vec3 {
        let x = if dir.x > 0.0 { self.max.x } else { self.min.x };
        let y = if dir.y > 0.0 { self.max.y } else { self.min.y };
        let z = if dir.z > 0.0 { self.max.z } else { self.min.z };

        vec3(x, y, z)
    }
}

pub type DynShape = Arc<dyn Shape + Send + Sync>;

impl<'a, T> Shape for &'a T
where
    T: Shape + ?Sized,
{
    fn support(&self, dir: Vec3) -> Vec3 {
        (*self).support(dir)
    }
}
