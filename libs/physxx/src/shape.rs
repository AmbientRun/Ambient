use std::ffi::c_void;

use num_traits::FromPrimitive;

use crate::{
    AsPxBase, PxBaseRef, PxGeometry, PxGeometryHolder, PxGeometryType, PxMaterial, PxPhysicsRef, PxRigidActorRef, PxTransform, PxUserData
};

bitflags! {
    pub struct PxShapeFlag: u8 {
        const SCENE_QUERY_SHAPE = physx_sys::PxShapeFlag::eSCENE_QUERY_SHAPE as u8;
        const SIMULATION_SHAPE = physx_sys::PxShapeFlag::eSIMULATION_SHAPE as u8;
        const TRIGGER_SHAPE = physx_sys::PxShapeFlag::eTRIGGER_SHAPE as u8;
        const VISUALIZATION = physx_sys::PxShapeFlag::eVISUALIZATION as u8;
    }
}
impl Default for PxShapeFlag {
    fn default() -> Self {
        Self::VISUALIZATION | Self::SIMULATION_SHAPE
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct PxShape(pub *mut physx_sys::PxShape);
impl PxShape {
    pub fn new(
        physics: PxPhysicsRef,
        geometry: &dyn PxGeometry,
        materials: &[&PxMaterial],
        is_exclusive: Option<bool>,
        shape_flags: Option<PxShapeFlag>,
    ) -> Self {
        let mats = materials.iter().map(|x| x.0).collect::<Vec<*mut physx_sys::PxMaterial>>();
        Self(unsafe {
            physx_sys::PxPhysics_createShape_mut_1(
                physics.0,
                geometry.as_geometry_ptr(),
                mats.as_ptr() as *const *mut physx_sys::PxMaterial,
                materials.len() as u16,
                is_exclusive.unwrap_or(false),
                physx_sys::PxShapeFlags { mBits: shape_flags.unwrap_or_default().bits },
            )
        })
    }
    pub(crate) fn from_ptr(ptr: *mut physx_sys::PxShape) -> Self {
        let mut s = Self(ptr);
        s.acquire_reference();
        s
    }
    pub(crate) fn acquire_reference(&mut self) {
        unsafe { physx_sys::PxShape_acquireReference_mut(self.0) }
    }
    pub fn get_geometry(&self) -> PxGeometryHolder {
        PxGeometryHolder(unsafe { physx_sys::PxShape_getGeometry(self.0) })
    }
    pub fn set_geometry(&self, geometry: &dyn PxGeometry) {
        unsafe { physx_sys::PxShape_setGeometry_mut(self.0, geometry.as_geometry_ptr()) }
    }
    pub fn get_geometry_type(&self) -> PxGeometryType {
        PxGeometryType::from_i32(unsafe { physx_sys::PxShape_getGeometryType(self.0) }).unwrap()
    }
    pub fn get_local_pose(&self) -> PxTransform {
        PxTransform(unsafe { physx_sys::PxShape_getLocalPose(self.0) })
    }
    pub fn set_local_pose(&self, pose: &PxTransform) {
        unsafe { physx_sys::PxShape_setLocalPose_mut(self.0, &pose.0 as *const physx_sys::PxTransform) }
    }
    pub fn get_global_pose(&self, actor: PxRigidActorRef) -> PxTransform {
        PxTransform(unsafe { physx_sys::PxShapeExt_getGlobalPose_mut(self.0, actor.0) })
    }
    pub fn get_actor(&self) -> Option<PxRigidActorRef> {
        unsafe {
            let p = physx_sys::PxShape_getActor(self.0);
            if p.is_null() {
                None
            } else {
                Some(PxRigidActorRef(p))
            }
        }
    }
    pub fn get_flags(&self) -> PxShapeFlag {
        PxShapeFlag::from_bits(unsafe { physx_sys::PxShape_getFlags(self.0) }.mBits).unwrap()
    }
    pub fn set_flag(&self, flag: PxShapeFlag, value: bool) {
        unsafe { physx_sys::PxShape_setFlag_mut(self.0, flag.bits as u32, value) }
    }
    pub fn set_flags(&self, flags: PxShapeFlag) {
        unsafe { physx_sys::PxShape_setFlags_mut(self.0, physx_sys::PxShapeFlags { mBits: flags.bits }) }
    }
}
impl AsPxBase for PxShape {
    fn as_base(&self) -> PxBaseRef {
        PxBaseRef(self.0 as _)
    }
}
impl PxUserData for PxShape {
    fn raw_user_data_mut(&self) -> &mut *mut c_void {
        unsafe { &mut (*self.0).userData }
    }
    fn raw_user_data(&self) -> &*mut c_void {
        unsafe { &(*self.0).userData }
    }
}
impl Clone for PxShape {
    fn clone(&self) -> Self {
        Self::from_ptr(self.0)
    }
}
impl Drop for PxShape {
    fn drop(&mut self) {
        unsafe {
            physx_sys::PxShape_release_mut(self.0);
        }
    }
}
unsafe impl Sync for PxShape {}
unsafe impl Send for PxShape {}
