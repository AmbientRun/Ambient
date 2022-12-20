use glam::Vec3;

pub const TOLERANCE: f32 = 0.01;

#[derive(Copy, PartialEq, Debug, Clone)]
pub struct Triangle {
    pub a: Vec3,
    pub b: Vec3,
    pub c: Vec3,
}

impl Triangle {
    pub fn new(a: Vec3, b: Vec3, c: Vec3) -> Self {
        Self { a, b, c }
    }
}

#[derive(Copy, PartialEq, Debug, Clone)]
pub struct Barycentric3 {
    pub u: f32,
    pub v: f32,
    pub w: f32,
}

impl Barycentric3 {
    /// Computes the barycentric coordinates of `p` onto the triangle `tri` oriented in 3D space.
    ///
    /// `p` may lie inside or outside the triangle
    pub fn from_point(p: Vec3, tri: Triangle) -> Self {
        let v0 = tri.b - tri.a;
        let v1 = tri.c - tri.a;
        let v2 = p - tri.a;

        let d00 = v0.dot(v0);
        let d01 = v0.dot(v1);
        let d11 = v1.dot(v1);
        let d20 = v2.dot(v0);
        let d21 = v2.dot(v1);
        let denom = d00 * d11 - d01 * d01;

        let v = (d11 * d20 - d01 * d21) / denom;
        let w = (d00 * d21 - d01 * d20) / denom;
        let u = 1.0 - v - w;

        Self { u, v, w }
    }

    /// Returns true if the barycenter lies inside the triangle
    #[inline]
    pub fn is_inside(&self) -> bool {
        self.u >= -TOLERANCE && self.v >= -TOLERANCE && self.w >= -TOLERANCE
    }

    /// Construct a pair of barycentric coordinates from a ray intersecting *within* the triangle
    /// `tri`.
    pub fn from_ray(origin: Vec3, dir: Vec3, tri: Triangle) -> Option<Self> {
        let ba = tri.b - tri.a;
        let ca = tri.c - tri.a;

        let normal = ba.cross(ca).normalize();
        let d = -tri.a.dot(normal);

        let u = -(origin.dot(normal) + d);
        let v = dir.dot(normal);
        let t = u / v;

        if (0.0..=1.0).contains(&t) {
            let point = origin + dir * t;
            let bary = Self::from_point(point, tri);
            if bary.is_inside() {
                return Some(bary);
            }
        }

        None
    }
}
