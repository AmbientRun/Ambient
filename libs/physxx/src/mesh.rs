use glam::Vec3;

use crate::{
    AsPxPtr, PxConvexMeshCookingResult, PxCookingRef, PxDefaultMemoryInputData,
    PxDefaultMemoryOutputStream, PxPhysicsRef, PxReferenceCounted,
};

bitflags! {
    pub struct PxConvexFlag: u32 {
        const _16_BIT_INDICES = physx_sys::PxConvexFlag::e16_BIT_INDICES;
        const CHECK_ZERO_AREA_TRIANGLES = physx_sys::PxConvexFlag::eCHECK_ZERO_AREA_TRIANGLES;
        const COMPUTE_CONVEX = physx_sys::PxConvexFlag::eCOMPUTE_CONVEX;
        const DISABLE_MESH_VALIDATION = physx_sys::PxConvexFlag::eDISABLE_MESH_VALIDATION;
        const FAST_INERTIA_COMPUTATION = physx_sys::PxConvexFlag::eFAST_INERTIA_COMPUTATION;
        const GPU_COMPATIBLE = physx_sys::PxConvexFlag::eGPU_COMPATIBLE;
        const PLANE_SHIFTING = physx_sys::PxConvexFlag::ePLANE_SHIFTING;
        const QUANTIZE_INPUT = physx_sys::PxConvexFlag::eQUANTIZE_INPUT;
        const SHIFT_VERTICES = physx_sys::PxConvexFlag::eSHIFT_VERTICES;
    }
}

pub struct PxConvexMeshDesc {
    pub points: Vec<Vec3>,
    pub indices: Option<Vec<u32>>,
    pub vertex_limit: Option<u16>,
    pub flags: Option<PxConvexFlag>,
}

#[derive(Debug)]
pub struct PxConvexMesh(pub *mut physx_sys::PxConvexMesh);
impl PxConvexMesh {
    pub fn new(physics: PxPhysicsRef, input: &impl AsPxPtr<*mut physx_sys::PxInputStream>) -> Self {
        unsafe {
            let mesh = physx_sys::PxPhysics_createConvexMesh_mut(physics.0, input.as_px_ptr());
            Self(mesh)
        }
    }
    pub fn from_desc(
        physics: PxPhysicsRef,
        cooking: PxCookingRef,
        desc: PxConvexMeshDesc,
    ) -> Result<Self, PxConvexMeshCookingResult> {
        let stream = PxDefaultMemoryOutputStream::new();
        let mut res = PxConvexMeshCookingResult::Success;
        if !cooking.cook_convex_mesh(&desc, &stream, &mut res) {
            Err(res)
        } else {
            let input = PxDefaultMemoryInputData::new(stream.get_data());
            Ok(Self::new(physics, &input))
        }
    }
    pub(crate) fn from_ptr(ptr: *mut physx_sys::PxConvexMesh) -> Self {
        let mut s = Self(ptr);
        s.acquire_reference();
        s
    }
    pub(crate) fn acquire_reference(&mut self) {
        unsafe { physx_sys::PxConvexMesh_acquireReference_mut(self.0) }
    }
}
impl PxReferenceCounted for PxConvexMesh {
    fn get_reference_count(&self) -> u32 {
        unsafe { physx_sys::PxConvexMesh_getReferenceCount(self.0) }
    }
}
impl Clone for PxConvexMesh {
    fn clone(&self) -> Self {
        Self::from_ptr(self.0)
    }
}
impl Drop for PxConvexMesh {
    fn drop(&mut self) {
        unsafe { physx_sys::PxConvexMesh_release_mut(self.0) }
    }
}
unsafe impl Sync for PxConvexMesh {}
unsafe impl Send for PxConvexMesh {}

bitflags! {
    pub struct PxMeshFlag: u32 {
        const _16_BIT_INDICES = physx_sys::PxMeshFlag::e16_BIT_INDICES;
        const FLIPNORMALS = physx_sys::PxMeshFlag::eFLIPNORMALS;
    }
}

pub struct PxTriangleMeshDesc {
    pub points: Vec<Vec3>,
    pub indices: Vec<u32>,
    pub flags: Option<PxMeshFlag>,
}

#[derive(Debug)]
pub struct PxTriangleMesh(pub *mut physx_sys::PxTriangleMesh);
impl PxTriangleMesh {
    pub fn new(physics: PxPhysicsRef, input: &impl AsPxPtr<*mut physx_sys::PxInputStream>) -> Self {
        unsafe {
            let mesh = physx_sys::PxPhysics_createTriangleMesh_mut(physics.0, input.as_px_ptr());
            Self(mesh)
        }
    }
    pub(crate) fn from_ptr(ptr: *mut physx_sys::PxTriangleMesh) -> Self {
        let mut s = Self(ptr);
        s.acquire_reference();
        s
    }
    pub(crate) fn acquire_reference(&mut self) {
        unsafe { physx_sys::PxTriangleMesh_acquireReference_mut(self.0) }
    }
    pub fn get_nb_vertices(&self) -> u32 {
        unsafe { physx_sys::PxTriangleMesh_getNbVertices(self.0) }
    }
    pub fn get_nb_triangles(&self) -> u32 {
        unsafe { physx_sys::PxTriangleMesh_getNbTriangles(self.0) }
    }
}
impl PxReferenceCounted for PxTriangleMesh {
    fn get_reference_count(&self) -> u32 {
        unsafe { physx_sys::PxTriangleMesh_getReferenceCount(self.0) }
    }
}
impl Clone for PxTriangleMesh {
    fn clone(&self) -> Self {
        Self::from_ptr(self.0)
    }
}
impl Drop for PxTriangleMesh {
    fn drop(&mut self) {
        unsafe { physx_sys::PxTriangleMesh_release_mut(self.0) }
    }
}
unsafe impl Sync for PxTriangleMesh {}
unsafe impl Send for PxTriangleMesh {}
