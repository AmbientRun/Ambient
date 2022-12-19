use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::{
    to_physx_vec3, AsPxPtr, PxConvexMeshDesc, PxFoundationRef, PxHeightField, PxHeightFieldDesc, PxPhysicsRef, PxTriangleMeshDesc, PX_PHYSICS_VERSION
};

pub struct PxCookingParams(pub physx_sys::PxCookingParams);
impl PxCookingParams {
    /// Create a new cooking params.
    pub fn new(physics: &PxPhysicsRef) -> Self {
        Self(unsafe { physx_sys::PxCookingParams_new(physics.get_tolerances_scale()) })
    }
}

#[derive(Debug, Clone, Copy, FromPrimitive)]
#[repr(u32)]
pub enum PxTriangleMeshCookingResult {
    Failure = physx_sys::PxTriangleMeshCookingResult::eFAILURE,
    LargeTriangle = physx_sys::PxTriangleMeshCookingResult::eLARGE_TRIANGLE,
    Success = physx_sys::PxTriangleMeshCookingResult::eSUCCESS,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
pub enum PxConvexMeshCookingResult {
    Failure = physx_sys::PxConvexMeshCookingResult::eFAILURE,
    PolygonsLimitReached = physx_sys::PxConvexMeshCookingResult::ePOLYGONS_LIMIT_REACHED,
    Success = physx_sys::PxConvexMeshCookingResult::eSUCCESS,
    ZeroAreaTestFailed = physx_sys::PxConvexMeshCookingResult::eZERO_AREA_TEST_FAILED,
}

#[derive(Clone, Copy)]
pub struct PxCookingRef(pub(crate) *mut physx_sys::PxCooking);
impl PxCookingRef {
    pub fn new(foundation: &PxFoundationRef, params: &PxCookingParams) -> Self {
        unsafe { Self(physx_sys::phys_PxCreateCooking(PX_PHYSICS_VERSION, foundation.0, &params.0 as *const physx_sys::PxCookingParams)) }
    }
    pub fn create_height_field(&self, physics: &PxPhysicsRef, desc: &PxHeightFieldDesc) -> PxHeightField {
        PxHeightField(unsafe {
            physx_sys::PxCooking_createHeightField(
                self.0,
                &desc.0 as *const physx_sys::PxHeightFieldDesc,
                physics.get_physics_insertion_callback(),
            )
        })
    }
    pub fn cook_triangle_mesh(
        &self,
        desc: &PxTriangleMeshDesc,
        stream: &impl AsPxPtr<*mut physx_sys::PxOutputStream>,
        res: &mut PxTriangleMeshCookingResult,
    ) -> bool {
        unsafe {
            // Lifetime of the points need to live outside of the physx_sys::PxTriangleMeshDesc, so we create the physx_sys::PxTriangleMeshDesc
            // here temporarily. It doesn't "cost" anything any since it's just a struct of pointers
            let mut px_desc = physx_sys::PxTriangleMeshDesc_new();
            let points = desc.points.iter().map(|p| to_physx_vec3(*p)).collect::<Vec<_>>();
            px_desc.points.count = points.len() as u32;
            px_desc.points.stride = std::mem::size_of::<physx_sys::PxVec3>() as u32;
            px_desc.points.data = points.as_ptr() as _;
            px_desc.triangles.count = desc.indices.len() as u32 / 3;
            px_desc.triangles.stride = 3 * std::mem::size_of::<u32>() as u32;
            px_desc.triangles.data = desc.indices.as_ptr() as _;
            if let Some(flags) = desc.flags {
                px_desc.flags = physx_sys::PxMeshFlags { mBits: flags.bits() as u16 };
            }

            let mut res_tmp = 0;
            let ret = physx_sys::PxCooking_cookTriangleMesh(self.0, &px_desc as _, stream.as_px_ptr(), &mut res_tmp);
            *res = PxTriangleMeshCookingResult::from_u32(res_tmp).unwrap();
            ret
        }
    }
    pub fn cook_convex_mesh(
        &self,
        desc: &PxConvexMeshDesc,
        stream: &impl AsPxPtr<*mut physx_sys::PxOutputStream>,
        res: &mut PxConvexMeshCookingResult,
    ) -> bool {
        unsafe {
            let mut px_desc = physx_sys::PxConvexMeshDesc_new();
            let points = desc.points.iter().map(|x| to_physx_vec3(*x)).collect::<Vec<_>>();
            px_desc.points.count = points.len() as u32;
            px_desc.points.stride = std::mem::size_of::<physx_sys::PxVec3>() as u32;
            px_desc.points.data = points.as_ptr() as _;
            if let Some(indices) = &desc.indices {
                px_desc.indices.count = indices.len() as u32;
                px_desc.indices.stride = std::mem::size_of::<u32>() as u32;
                px_desc.indices.data = indices.as_ptr() as _;
            }
            if let Some(vertex_limit) = desc.vertex_limit {
                px_desc.vertexLimit = vertex_limit;
            }
            if let Some(flags) = desc.flags {
                px_desc.flags = physx_sys::PxConvexFlags { mBits: flags.bits() as u16 };
            }
            if !physx_sys::PxConvexMeshDesc_isValid(&px_desc) {
                panic!("Invalid convex mesh desc");
            }
            let mut res_tmp = 0;
            let ret = physx_sys::PxCooking_cookConvexMesh(self.0, &px_desc, stream.as_px_ptr(), &mut res_tmp);
            *res = FromPrimitive::from_u32(res_tmp).unwrap();
            ret
        }
    }
    pub fn release(self) {
        unsafe { physx_sys::PxCooking_release_mut(self.0) }
    }
}
unsafe impl Sync for PxCookingRef {}
unsafe impl Send for PxCookingRef {}
