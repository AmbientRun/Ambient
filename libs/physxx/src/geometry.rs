use glam::{Quat, Vec3};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use physx_sys::PxGeometryQuery_computePenetration_mut;

use crate::{
    to_glam_quat, to_glam_vec3, to_physx_quat, to_physx_vec3, PxConvexMesh, PxHeightField,
    PxTransform, PxTriangleMesh,
};

#[repr(i32)]
#[derive(Debug, FromPrimitive, PartialEq, Eq)]
pub enum PxGeometryType {
    Box = physx_sys::PxGeometryType::eBOX,
    Capsule = physx_sys::PxGeometryType::eCAPSULE,
    ConvexMesh = physx_sys::PxGeometryType::eCONVEXMESH,
    GeometryCount = physx_sys::PxGeometryType::eGEOMETRY_COUNT,
    HEIGHTFIELD = physx_sys::PxGeometryType::eHEIGHTFIELD,
    INVALID = physx_sys::PxGeometryType::eINVALID,
    PLANE = physx_sys::PxGeometryType::ePLANE,
    SPHERE = physx_sys::PxGeometryType::eSPHERE,
    TRIANGLEMESH = physx_sys::PxGeometryType::eTRIANGLEMESH,
}

pub trait PxGeometry {
    fn as_geometry_ptr(&self) -> *const physx_sys::PxGeometry;

    fn get_type(&self) -> PxGeometryType {
        unsafe {
            PxGeometryType::from_i32(physx_sys::PxGeometry_getType(self.as_geometry_ptr())).unwrap()
        }
    }
}

#[derive(Clone)]
pub struct PxPlaneGeometry(physx_sys::PxPlaneGeometry);
impl PxPlaneGeometry {
    pub fn new() -> Self {
        Self(unsafe { physx_sys::PxPlaneGeometry_new() })
    }
}
impl PxGeometry for PxPlaneGeometry {
    fn as_geometry_ptr(&self) -> *const physx_sys::PxGeometry {
        &self.0 as *const physx_sys::PxPlaneGeometry as *const physx_sys::PxGeometry
    }
}

#[derive(Clone)]
pub struct PxSphereGeometry(physx_sys::PxSphereGeometry);
impl PxSphereGeometry {
    pub fn new(radius: f32) -> Self {
        Self(unsafe { physx_sys::PxSphereGeometry_new_1(radius) })
    }
    pub fn radius(&self) -> f32 {
        self.0.radius
    }
}
impl PxGeometry for PxSphereGeometry {
    fn as_geometry_ptr(&self) -> *const physx_sys::PxGeometry {
        &self.0 as *const physx_sys::PxSphereGeometry as *const physx_sys::PxGeometry
    }
}

#[derive(Clone)]
pub struct PxBoxGeometry(physx_sys::PxBoxGeometry);
impl PxBoxGeometry {
    pub fn new(half_x: f32, half_y: f32, half_z: f32) -> Self {
        Self(unsafe { physx_sys::PxBoxGeometry_new_1(half_x, half_y, half_z) })
    }
    pub fn half_extents(&self) -> Vec3 {
        to_glam_vec3(&self.0.halfExtents)
    }
}
impl PxGeometry for PxBoxGeometry {
    fn as_geometry_ptr(&self) -> *const physx_sys::PxGeometry {
        &self.0 as *const physx_sys::PxBoxGeometry as *const physx_sys::PxGeometry
    }
}

#[derive(Clone)]
pub struct PxCapsuleGeometry(physx_sys::PxCapsuleGeometry);
impl PxCapsuleGeometry {
    pub fn new(radius: f32, half_height: f32) -> Self {
        Self(unsafe { physx_sys::PxCapsuleGeometry_new_1(radius, half_height) })
    }
}
impl PxGeometry for PxCapsuleGeometry {
    fn as_geometry_ptr(&self) -> *const physx_sys::PxGeometry {
        &self.0 as *const physx_sys::PxCapsuleGeometry as *const physx_sys::PxGeometry
    }
}

#[derive(Clone)]
pub struct PxMeshScale(physx_sys::PxMeshScale);
impl PxMeshScale {
    pub fn identity() -> Self {
        PxMeshScale(unsafe { physx_sys::PxMeshScale_new() })
    }
    pub fn from_scale_uniform(scale: f32) -> Self {
        PxMeshScale(unsafe { physx_sys::PxMeshScale_new_1(scale) })
    }
    pub fn from_scale(scale: Vec3) -> Self {
        PxMeshScale(unsafe { physx_sys::PxMeshScale_new_2(&to_physx_vec3(scale)) })
    }
    pub fn from_scale_rotation(scale: Vec3, rotation: Quat) -> Self {
        PxMeshScale(unsafe {
            physx_sys::PxMeshScale_new_3(&to_physx_vec3(scale), &to_physx_quat(rotation))
        })
    }
    pub fn scale(&self) -> Vec3 {
        to_glam_vec3(&self.0.scale)
    }
    pub fn rotation(&self) -> Quat {
        to_glam_quat(self.0.rotation)
    }
}

bitflags! {
    pub struct PxConvexMeshGeometryFlag: u32 {
        const TIGHT_BOUNDS = physx_sys::PxConvexMeshGeometryFlag::eTIGHT_BOUNDS;
    }
}

#[derive(Clone)]
pub struct PxConvexMeshGeometry(physx_sys::PxConvexMeshGeometry);
impl PxConvexMeshGeometry {
    pub fn new(
        mesh: &PxConvexMesh,
        scaling: Option<PxMeshScale>,
        flags: Option<PxConvexMeshGeometryFlag>,
    ) -> Self {
        let scaling = scaling.unwrap_or_else(PxMeshScale::identity);
        let flags = flags.unwrap_or(PxConvexMeshGeometryFlag::empty());
        Self(unsafe {
            physx_sys::PxConvexMeshGeometry_new_1(
                mesh.0,
                &scaling.0 as _,
                physx_sys::PxConvexMeshGeometryFlags {
                    mBits: flags.bits as u8,
                },
            )
        })
    }
    pub fn scale(&self) -> PxMeshScale {
        PxMeshScale(self.0.scale)
    }
    pub fn mesh(&self) -> PxConvexMesh {
        PxConvexMesh::from_ptr(self.0.convexMesh)
    }
    pub fn flags(&self) -> PxConvexMeshGeometryFlag {
        PxConvexMeshGeometryFlag::from_bits(self.0.meshFlags.mBits as u32).unwrap()
    }
    pub fn is_valid(&self) -> bool {
        unsafe { physx_sys::PxConvexMeshGeometry_isValid(&self.0 as _) }
    }
}
impl PxGeometry for PxConvexMeshGeometry {
    fn as_geometry_ptr(&self) -> *const physx_sys::PxGeometry {
        &self.0 as *const physx_sys::PxConvexMeshGeometry as *const physx_sys::PxGeometry
    }
}

bitflags! {
    pub struct PxMeshGeometryFlag: u32 {
        const DOUBLE_SIDED = physx_sys::PxMeshGeometryFlag::eDOUBLE_SIDED;
    }
}

#[derive(Clone)]
pub struct PxTriangleMeshGeometry(physx_sys::PxTriangleMeshGeometry);
impl PxTriangleMeshGeometry {
    pub fn new(
        mesh: &PxTriangleMesh,
        scaling: Option<PxMeshScale>,
        flags: Option<PxMeshGeometryFlag>,
    ) -> Self {
        let scaling = scaling.unwrap_or_else(PxMeshScale::identity);
        let flags = flags.unwrap_or(PxMeshGeometryFlag::empty());
        Self(unsafe {
            physx_sys::PxTriangleMeshGeometry_new_1(
                mesh.0,
                &scaling.0 as _,
                physx_sys::PxMeshGeometryFlags {
                    mBits: flags.bits as u8,
                },
            )
        })
    }
    pub fn scale(&self) -> PxMeshScale {
        PxMeshScale(self.0.scale)
    }
    pub fn mesh(&self) -> PxTriangleMesh {
        PxTriangleMesh::from_ptr(self.0.triangleMesh)
    }
    pub fn flags(&self) -> PxMeshGeometryFlag {
        PxMeshGeometryFlag::from_bits(self.0.meshFlags.mBits as u32).unwrap()
    }
    pub fn is_valid(&self) -> bool {
        unsafe { physx_sys::PxTriangleMeshGeometry_isValid(&self.0 as _) }
    }
}
impl PxGeometry for PxTriangleMeshGeometry {
    fn as_geometry_ptr(&self) -> *const physx_sys::PxGeometry {
        &self.0 as *const physx_sys::PxTriangleMeshGeometry as *const physx_sys::PxGeometry
    }
}

pub struct PxHeightFieldGeometry(physx_sys::PxHeightFieldGeometry);
impl PxHeightFieldGeometry {
    pub fn new(
        height_field: &mut PxHeightField,
        height_scale: f32,
        row_scale: f32,
        column_scale: f32,
    ) -> Self {
        Self(unsafe {
            physx_sys::PxHeightFieldGeometry_new_1(
                height_field.0,
                physx_sys::PxMeshGeometryFlags { mBits: 0 },
                height_scale,
                row_scale,
                column_scale,
            )
        })
    }
}
impl PxGeometry for PxHeightFieldGeometry {
    fn as_geometry_ptr(&self) -> *const physx_sys::PxGeometry {
        &self.0 as *const physx_sys::PxHeightFieldGeometry as *const physx_sys::PxGeometry
    }
}

pub struct PxGeometryHolder(pub physx_sys::PxGeometryHolder);
impl PxGeometryHolder {
    pub fn as_box(&self) -> Option<PxBoxGeometry> {
        if self.get_type() != PxGeometryType::Box {
            return None;
        }
        unsafe { Some(PxBoxGeometry(*physx_sys::PxGeometryHolder_box(&self.0))) }
    }
    pub fn as_sphere(&self) -> Option<PxSphereGeometry> {
        if self.get_type() != PxGeometryType::SPHERE {
            return None;
        }
        unsafe {
            Some(PxSphereGeometry(*physx_sys::PxGeometryHolder_sphere(
                &self.0,
            )))
        }
    }
    pub fn as_convex_mesh(&self) -> Option<PxConvexMeshGeometry> {
        if self.get_type() != PxGeometryType::ConvexMesh {
            return None;
        }
        unsafe {
            Some(PxConvexMeshGeometry(
                *physx_sys::PxGeometryHolder_convexMesh(&self.0),
            ))
        }
    }
    pub fn as_triangle_mesh(&self) -> Option<PxTriangleMeshGeometry> {
        if self.get_type() != PxGeometryType::TRIANGLEMESH {
            return None;
        }
        unsafe {
            Some(PxTriangleMeshGeometry(
                *physx_sys::PxGeometryHolder_triangleMesh(&self.0),
            ))
        }
    }
}
impl PxGeometry for PxGeometryHolder {
    fn as_geometry_ptr(&self) -> *const physx_sys::PxGeometry {
        unsafe { physx_sys::PxGeometryHolder_any(&self.0 as *const physx_sys::PxGeometryHolder) }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeometryIntersection {
    pub dir: Vec3,
    pub depth: f32,
}

/// Perform a penetration test between two geometries
pub fn compute_penetration(
    geom0: &dyn PxGeometry,
    pose0: &PxTransform,
    geom1: &dyn PxGeometry,
    pose1: &PxTransform,
) -> Option<GeometryIntersection> {
    unsafe {
        let mut dir = to_physx_vec3(Default::default());
        let mut depth = Default::default();
        if PxGeometryQuery_computePenetration_mut(
            &mut dir as *mut _,
            &mut depth as *mut _,
            geom0.as_geometry_ptr(),
            &pose0.0 as *const _,
            geom1.as_geometry_ptr(),
            &pose1.0 as *const _,
        ) {
            Some(GeometryIntersection {
                dir: to_glam_vec3(&dir),
                depth,
            })
        } else {
            None
        }
    }
}
