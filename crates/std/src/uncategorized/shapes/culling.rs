use super::*;

pub trait Cullable<T> {
    fn cull(&self, bounding: &T) -> CullResult;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CullResult {
    AContainsB,
    BContainsA,
    Intersect,
    Outside,
}
impl CullResult {
    fn invert(&self) -> CullResult {
        match *self {
            CullResult::AContainsB => CullResult::BContainsA,
            CullResult::BContainsA => CullResult::AContainsB,
            CullResult::Intersect => CullResult::Intersect,
            CullResult::Outside => CullResult::Outside,
        }
    }
}

impl Cullable<AABB> for AABB {
    fn cull(&self, bounding: &AABB) -> CullResult {
        let hw1 = (self.max - self.min) / 2.0;
        let hw2 = (bounding.max - bounding.min) / 2.0;
        let c1 = self.min + hw1;
        let c2 = bounding.min + hw2;
        let d = c1 - c2;
        let d = vec3(d.x.abs(), d.y.abs(), d.z.abs());
        let e = hw1 + hw2;
        if d.x > e.x {
            return CullResult::Outside;
        }
        if d.y > e.y {
            return CullResult::Outside;
        }
        if d.z > e.z {
            return CullResult::Outside;
        }
        if d.x + hw2.x < hw1.x && d.y + hw2.y < hw1.y && d.z + hw2.x < hw1.z {
            return CullResult::AContainsB;
        }
        CullResult::Intersect
    }
}

impl Cullable<Plane> for AABB {
    fn cull(&self, plane: &Plane) -> CullResult {
        let mut min = Vec3::ZERO;
        let mut max = Vec3::ZERO;
        if plane.normal.x > 0.0 {
            min.x = self.min.x;
            max.x = self.max.x;
        } else {
            min.x = self.max.x;
            max.x = self.min.x;
        }

        if plane.normal.y > 0.0 {
            min.y = self.min.y;
            max.y = self.max.y;
        } else {
            min.y = self.max.y;
            max.y = self.min.y;
        }

        if plane.normal.z > 0.0 {
            min.z = self.min.z;
            max.z = self.max.z;
        } else {
            min.z = self.max.z;
            max.z = self.min.z;
        }

        if min.dot(plane.normal) + plane.distance > 0.0 {
            return CullResult::Outside;
        }
        if max.dot(plane.normal) + plane.distance < 0.0 {
            return CullResult::BContainsA;
        }
        CullResult::Intersect
    }
}

impl Cullable<AABB> for Plane {
    fn cull(&self, aabb: &AABB) -> CullResult {
        aabb.cull(self).invert()
    }
}

impl Cullable<Plane> for Sphere {
    fn cull(&self, plane: &Plane) -> CullResult {
        let dist = plane.distance(self.center);
        if dist < -self.radius {
            CullResult::Outside
        } else if dist < self.radius {
            CullResult::Intersect
        } else {
            CullResult::BContainsA
        }
    }
}

impl<T: Cullable<Plane>> Cullable<Frustum> for T {
    fn cull(&self, frustum: &Frustum) -> CullResult {
        if frustum.planes.is_empty() {
            return CullResult::Outside;
        }

        let mut res = CullResult::BContainsA;
        for p in &frustum.planes {
            let r = self.cull(p);
            if r == CullResult::Outside {
                return CullResult::Outside;
            }
            if r == CullResult::Intersect {
                res = CullResult::Intersect;
            }
        }
        res
    }
}
impl<T: Cullable<Frustum>> Cullable<T> for Frustum {
    fn cull(&self, bounding: &T) -> CullResult {
        bounding.cull(self).invert()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_plane_sphere_culling() {
        let _plane = Plane::new(Vec3::Z, 1.);
        // assert_eq!(BoundingSphere::new(Vec3::Z * 3., 1.).cull(&plane), CullResult::Outside);
        // assert_eq!(BoundingSphere::new(Vec3::Z * 1., 1.).cull(&plane), CullResult::Intersect);
        // assert_eq!(BoundingSphere::new(Vec3::Z * -3., 1.).cull(&plane), CullResult::AContainsB);
    }

    #[test]
    fn test_frustum_aabb_culling() {
        let eye = vec3(4.0, 4.0, 4.0);
        let center = vec3(0.0, 0.0, 0.0);
        let up = vec3(0.0, 0.0, 1.0);
        let cam = Mat4::perspective_lh(1.0, 1.0, 1.0, 10.0) * Mat4::look_at_lh(eye, center, up);
        let _ = Frustum::from_inv_projection_view(cam.inverse()).unwrap();
        // assert_eq!(f.cull(&AABB { min: Vec3::ZERO, max: vec3(0.1, 0.1, 0.1) }), CullResult::AContainsB);
        // assert_eq!(f.cull(&AABB { min: vec3(-50.0, 0.0, 0.0), max: vec3(-50.1, 0.1, 0.1) }), CullResult::Outside);
        // assert_eq!(f.cull(&AABB { min: vec3(0.0, 0.0, 0.0), max: vec3(30.0, 30.0, 30.0) }), CullResult::Intersect);
    }
}
// impl Cullable<Bounding> for Bounding {
//     fn cull(&self, bounding: &Bounding) -> CullResult {
//         match self {
//             &Bounding::AABB(ref a) => {
//                 match bounding {
//                     &Bounding::AABB(ref b) => a.cull(b),
//                     &Bounding::Plane(ref b) => a.cull(b),
//                     &Bounding::Frustum(ref b) => a.cull(b),
//                     _ => unimplemented!()
//                 }
//             },
//             &Bounding::Plane(ref a) => {
//                 match bounding {
//                     &Bounding::AABB(ref b) => a.cull(b),
//                     _ => unimplemented!()
//                 }
//             },
//             &Bounding::Frustum(ref a) => {
//                 match bounding {
//                     &Bounding::AABB(ref b) => a.cull(b),
//                     _ => unimplemented!()
//                 }
//             },
//             &Bounding::TransformedBounding(ref transformed) => {
//                 transformed.transformed_bounding.cull(bounding)
//             }
//         }
//     }
// }
