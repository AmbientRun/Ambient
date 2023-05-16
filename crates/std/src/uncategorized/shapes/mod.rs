use glam::{vec3, DVec3, Mat4, Vec3};

mod culling;
mod shape;
pub use culling::*;
pub use shape::*;

#[derive(PartialEq, Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct Cuboid {
    pub min: Vec3,
    pub max: Vec3,
}

impl FromIterator<AABB> for AABB {
    fn from_iter<T: IntoIterator<Item = AABB>>(iter: T) -> Self {
        iter.into_iter().reduce(|acc, v| acc.union(&v)).unwrap_or_default()
    }
}

impl Cuboid {
    pub const ZERO: AABB = AABB { min: Vec3::ZERO, max: Vec3::ZERO };

    pub fn unions(cubes: &[Self]) -> Option<Self> {
        if !cubes.is_empty() {
            let (&combined, cubes) = cubes.split_first().unwrap();
            let mut combined = combined;
            for aabb in cubes {
                combined = aabb.union(&combined);
            }
            Some(combined)
        } else {
            None
        }
    }

    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn new_invalid() -> Self {
        Self {
            min: Vec3::splat(f32::MAX),
            max: Vec3::splat(-f32::MAX),
        }
    }

    pub fn take_point(&mut self, point: Vec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    pub fn from_points(points: &[Vec3]) -> Self {
        let mut aabb = Self::new_invalid();
        for &point in points {
            aabb.take_point(point);
        }
        aabb
    }

    fn radius(&self) -> f32 {
        (self.extent()).length()
    }

    pub fn extent(&self) -> Vec3 {
        (self.max - self.min) / 2.
    }

    pub fn to_sphere(&self) -> Sphere {
        Sphere { center: self.center(), radius: self.radius() }
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) / 2.0
    }

    pub fn to_box(&self) -> BoundingBox {
        BoundingBox {
            points: vec![
                vec3(self.min.x, self.min.y, self.min.z),
                vec3(self.min.x, self.min.y, self.max.z),
                vec3(self.min.x, self.max.y, self.min.z),
                vec3(self.min.x, self.max.y, self.max.z),
                vec3(self.max.x, self.min.y, self.min.z),
                vec3(self.max.x, self.min.y, self.max.z),
                vec3(self.max.x, self.max.y, self.min.z),
                vec3(self.max.x, self.max.y, self.max.z),
            ],
        }
    }

    pub fn transform(&self, mat: &Mat4) -> BoundingBox {
        self.to_box().transform(mat)
    }

    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    pub fn translate(&self, pos: Vec3) -> Self {
        Self { min: self.min + pos, max: self.max + pos }
    }

    pub fn union(&self, other: &AABB) -> Self {
        Self { min: self.min.min(other.min), max: self.max.max(other.max) }
    }

    pub fn intersect_aabb(&self, other: &AABB) -> bool {
        (self.min.x <= other.max.x && self.max.x >= other.min.x)
            && (self.min.y <= other.max.y && self.max.y >= other.min.y)
            && (self.min.z <= other.max.z && self.max.z >= other.min.z)
    }
}

pub type AABB = Cuboid;

impl From<(Vec3, Vec3)> for Cuboid {
    fn from((min, max): (Vec3, Vec3)) -> Self {
        Self::new(min, max)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }

    pub fn transform(&self, mat: &Mat4) -> Self {
        let (scale, _, _) = mat.to_scale_rotation_translation();
        let scale = scale.x.max(scale.y.max(scale.z));
        Self { center: mat.project_point3(self.center), radius: self.radius * scale }
    }

    pub fn to_aabb(&self) -> AABB {
        AABB { min: self.center - self.radius, max: self.center + self.radius }
    }
}

#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub points: Vec<Vec3>,
}
impl BoundingBox {
    pub fn transform(&self, mat: &Mat4) -> Self {
        Self { points: self.points.iter().map(|&p| mat.project_point3(p)).collect() }
    }
    pub fn to_aabb(&self) -> AABB {
        let mut aabb = AABB { min: self.points[0], max: self.points[0] };
        for p in self.points.iter().skip(1) {
            aabb.min.x = aabb.min.x.min(p.x);
            aabb.min.y = aabb.min.y.min(p.y);
            aabb.min.z = aabb.min.z.min(p.z);

            aabb.max.x = aabb.max.x.max(p.x);
            aabb.max.y = aabb.max.y.max(p.y);
            aabb.max.z = aabb.max.z.max(p.z);
        }
        aabb
    }
    pub fn to_sphere(&self) -> Sphere {
        let center = self.points.iter().sum::<Vec3>() / self.points.len() as f32;
        let mut max_dist = 0.;
        for p in self.points.iter() {
            let dist = (*p - center).length();
            if dist > max_dist {
                max_dist = dist;
            }
        }
        Sphere { center, radius: max_dist }
    }
}

macro_rules! impl_plane {
    ($t:ident, $vec3:ident, $f32:ident) => {
        #[repr(C)]
        #[derive(Debug, Clone, Copy, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable, serde::Serialize, serde::Deserialize)]
        pub struct $t {
            pub normal: $vec3,
            pub distance: $f32,
        }
        impl $t {
            pub fn new(normal: $vec3, distance: $f32) -> Self {
                Self { normal, distance }
            }
            pub fn zero() -> Self {
                Self::new($vec3::ZERO, 0.)
            }
            pub fn from_points(a: $vec3, b: $vec3, c: $vec3) -> Option<Self> {
                let n = (b - a).cross(c - a);

                if n.length() == 0. {
                    None
                } else {
                    Self::from_normal_and_point(n, a)
                }
            }
            pub fn from_normal_and_point(normal: $vec3, point: $vec3) -> Option<Self> {
                let n = normal.normalize_or_zero();
                let d = -point.dot(n);
                if d.is_nan() {
                    None
                } else {
                    Some(Self::new(n, d))
                }
            }
            pub fn distance(&self, point: $vec3) -> $f32 {
                self.normal.dot(point) + self.distance
            }
            pub fn flip(&mut self) {
                self.normal = -self.normal;
                self.distance = -self.distance;
            }
            pub fn flipped(&self) -> Self {
                Self { normal: -self.normal, distance: -self.distance }
            }
        }
    };
}

impl_plane!(Plane, Vec3, f32);
impl_plane!(DPlane, DVec3, f64);

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Frustum {
    pub planes: [Plane; 6],
}
impl Frustum {
    pub fn from_inv_projection_view(inv_projection_view: Mat4) -> Option<Frustum> {
        let points = vec![
            inv_projection_view.project_point3(vec3(-1.0, -1.0, -1.0)),
            inv_projection_view.project_point3(vec3(-1.0, -1.0, 1.0)),
            inv_projection_view.project_point3(vec3(-1.0, 1.0, -1.0)),
            inv_projection_view.project_point3(vec3(-1.0, 1.0, 1.0)),
            inv_projection_view.project_point3(vec3(1.0, -1.0, -1.0)),
            inv_projection_view.project_point3(vec3(1.0, -1.0, 1.0)),
            inv_projection_view.project_point3(vec3(1.0, 1.0, -1.0)),
            inv_projection_view.project_point3(vec3(1.0, 1.0, 1.0)),
        ];
        Some(Frustum {
            planes: [
                Plane::from_points(points[0], points[2], points[1])?, // left
                Plane::from_points(points[4], points[5], points[6])?, // right
                Plane::from_points(points[0], points[1], points[4])?, // bottom
                Plane::from_points(points[2], points[6], points[3])?, // top
                Plane::from_points(points[0], points[4], points[2])?, // back
                Plane::from_points(points[1], points[3], points[5])?, // front
            ],
        })
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable, serde::Serialize, serde::Deserialize)]
pub struct Ray {
    pub origin: Vec3,
    pub dir: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, dir: Vec3) -> Self {
        Self { origin, dir }
    }
    pub fn transform(&self, transform: Mat4) -> Self {
        Self { origin: transform.project_point3(self.origin), dir: transform.transform_vector3(self.dir) }
    }
}

pub trait RayIntersectable {
    /// Returns the distance along the ray where the intersection is
    fn ray_intersect(&self, ray: Ray) -> Option<f32>;
}

impl RayIntersectable for Plane {
    fn ray_intersect(&self, ray: Ray) -> Option<f32> {
        let denom = self.normal.dot(ray.dir);
        if denom.is_normal() {
            let t = (-self.normal * self.distance - ray.origin).dot(self.normal) / denom;
            if t >= 0. {
                return Some(t);
            }
        }
        None
    }
}

impl RayIntersectable for AABB {
    fn ray_intersect(&self, ray: Ray) -> Option<f32> {
        let mut tmin = f32::NEG_INFINITY;
        let mut tmax = f32::INFINITY;

        for i in 0..3 {
            if ray.dir[i] != 0.0 {
                let t1 = (self.min[i] - ray.origin[i]) / ray.dir[i];
                let t2 = (self.max[i] - ray.origin[i]) / ray.dir[i];

                tmin = tmin.max(t1.min(t2));
                tmax = tmax.min(t1.max(t2));
            } else if ray.origin[i] <= self.min[i] || ray.origin[i] >= self.max[i] {
                return None;
            }
        }

        if tmax > tmin && tmax > 0.0 {
            Some(tmin)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ray_plane_intersection() {
        let plane = Plane::from_points(vec3(0., 0., 1.), vec3(1., 0., 1.), vec3(0., 1., 1.)).unwrap();
        let ray = Ray { origin: Vec3::Z * 10., dir: -Vec3::Z };
        assert_eq!(plane.ray_intersect(ray), Some(9.));
    }

    #[test]
    fn test_ray_plane_intersection2() {
        let plane = Plane::from_points(vec3(0., 0., 1.), vec3(1., 0., 1.), vec3(0., 1., 1.)).unwrap();
        let ray = Ray { origin: vec3(1., 1., 2.), dir: (-Vec3::ONE).normalize() };
        let hit = plane.ray_intersect(ray).unwrap();
        let p = ray.origin + ray.dir * hit;
        assert_eq!(p, Vec3::Z);
    }

    #[test]
    fn test_ray_aabb_intersection() {
        let ray = Ray { origin: vec3(500., 500., -1.), dir: Vec3::Z };
        let aabb = AABB { min: Vec3::ZERO, max: vec3(200., 200., 0.01) };
        let transform = Mat4::from_translation(vec3(400., 400., 0.));
        let transformed_ray = ray.transform(transform.inverse());
        assert_eq!(transformed_ray, Ray { origin: vec3(100., 100., -1.), dir: Vec3::Z });
        assert_eq!(aabb.ray_intersect(transformed_ray).unwrap(), 1.);
    }
}
