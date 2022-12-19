use std::ptr::null_mut;

use glam::{DVec3, Vec3};
use physx_sys::PxControllerCollisionFlag::*;

use crate::{to_glam_vec3, to_glam_vec3_f64, to_physx_vec3, to_physx_vec3_f64, PxMaterial, PxRigidDynamicRef, PxSceneRef};

#[derive(Clone, Copy)]
pub struct PxControllerManagerRef(*mut physx_sys::PxControllerManager);
impl PxControllerManagerRef {
    pub fn new(scene: &PxSceneRef, locking_enabled: bool) -> Self {
        Self(unsafe { physx_sys::phys_PxCreateControllerManager(scene.0, locking_enabled) })
    }
    pub fn create_controller(&self, desc: &PxControllerDesc) -> PxControllerRef {
        let obj = desc.to_physx();
        PxControllerRef(unsafe { physx_sys::PxControllerManager_createController_mut(self.0, obj.as_ptr()) })
    }
    pub fn create_obstacle_context(&self) -> PxObstacleContext {
        PxObstacleContext(unsafe { physx_sys::PxControllerManager_createObstacleContext_mut(self.0) })
    }
    pub fn release(&mut self) {
        unsafe { physx_sys::PxControllerManager_release_mut(self.0) }
    }
}
impl std::fmt::Debug for PxControllerManagerRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ControllerManager")
    }
}
unsafe impl Sync for PxControllerManagerRef {}
unsafe impl Send for PxControllerManagerRef {}

#[derive(Debug)]
pub enum PxControllerShapeDesc {
    Capsule { radius: f32, height: f32 },
}

#[derive(Debug)]
pub struct PxControllerDesc {
    pub position: DVec3,
    pub up_direction: Vec3,
    pub slope_limit: f32,
    pub invisible_wall_height: f32,
    pub max_jump_height: f32,
    pub contact_offset: f32,
    pub step_offset: f32,
    pub density: f32,
    pub scale_coeff: f32,
    pub volume_growth: f32,
    // pub reportCallback: *mut PxUserControllerHitReport,
    // pub behaviorCallback: *mut PxControllerBehaviorCallback,
    // pub non_walkable_mode: u32,
    pub material: PxMaterial,
    // pub register_deletion_listener: bool,
    // pub userData: *mut c_void,
    // pub mType: u32,
    pub shape: PxControllerShapeDesc,
}
impl PxControllerDesc {
    pub fn new(shape: PxControllerShapeDesc, material: PxMaterial) -> Self {
        let obj = match &shape {
            PxControllerShapeDesc::Capsule { .. } => unsafe {
                PxControllerDescRef::Capsule(physx_sys::PxCapsuleControllerDesc_new_alloc())
            },
        };
        let p = unsafe { &*obj.as_ptr() };
        Self {
            position: to_glam_vec3_f64(&p.position),
            up_direction: to_glam_vec3(&p.upDirection),
            slope_limit: p.slopeLimit,
            invisible_wall_height: p.invisibleWallHeight,
            max_jump_height: p.maxJumpHeight,
            contact_offset: p.contactOffset,
            step_offset: p.stepOffset,
            density: p.density,
            scale_coeff: p.scaleCoeff,
            volume_growth: p.volumeGrowth,
            shape,
            material,
        }
    }
    fn to_physx(&self) -> PxControllerDescRef {
        let obj = match self.shape {
            PxControllerShapeDesc::Capsule { radius, height } => unsafe {
                let p = physx_sys::PxCapsuleControllerDesc_new_alloc();
                (*p).radius = radius;
                (*p).height = height;
                PxControllerDescRef::Capsule(p)
            },
        };
        let p = unsafe { &mut *obj.as_ptr() };
        p.position = to_physx_vec3_f64(self.position);
        p.upDirection = to_physx_vec3(self.up_direction);
        p.slopeLimit = self.slope_limit;
        p.invisibleWallHeight = self.invisible_wall_height;
        p.maxJumpHeight = self.max_jump_height;
        p.contactOffset = self.contact_offset;
        p.stepOffset = self.step_offset;
        p.density = self.density;
        p.scaleCoeff = self.scale_coeff;
        p.volumeGrowth = self.volume_growth;
        p.material = self.material.0;
        obj
    }
    pub fn is_valid(&self) -> bool {
        unsafe { physx_sys::PxControllerDesc_isValid(self.to_physx().as_ptr()) }
    }
}

enum PxControllerDescRef {
    Capsule(*mut physx_sys::PxCapsuleControllerDesc),
}
impl PxControllerDescRef {
    fn as_ptr(&self) -> *mut physx_sys::PxControllerDesc {
        match self {
            PxControllerDescRef::Capsule(p) => (*p) as *mut physx_sys::PxControllerDesc,
        }
    }
}
impl Drop for PxControllerDescRef {
    fn drop(&mut self) {
        match self {
            PxControllerDescRef::Capsule(p) => unsafe { physx_sys::PxCapsuleControllerDesc_delete(*p) },
        }
    }
}
unsafe impl Sync for PxControllerDescRef {}
unsafe impl Send for PxControllerDescRef {}

// trait AsControllerDesc {
//     fn as_controller_desc_ptr(&self) -> *mut PxControllerDesc;
// }

// struct CapsuleControllerDesc(*mut PxCapsuleControllerDesc);
// impl AsControllerDesc for CapsuleControllerDesc {
//     fn as_controller_desc_ptr(&self) -> *mut PxControllerDesc {
//         self.0 as *mut PxControllerDesc
//     }
// }
// impl Drop for CapsuleControllerDesc {
//     fn drop(&mut self) {
//         unsafe { PxCapsuleControllerDesc_delete(self.0) }
//     }
// }

pub struct PxControllerFilters(physx_sys::PxControllerFilters);
impl PxControllerFilters {
    pub fn new() -> Self {
        Self(unsafe { physx_sys::PxControllerFilters_new(null_mut(), null_mut(), null_mut()) })
    }
}

pub struct PxObstacleContext(*mut physx_sys::PxObstacleContext);
impl PxObstacleContext {}
impl Drop for PxObstacleContext {
    fn drop(&mut self) {
        unsafe { physx_sys::PxObstacleContext_release_mut(self.0) }
    }
}
unsafe impl Sync for PxObstacleContext {}
unsafe impl Send for PxObstacleContext {}

bitflags! {
    pub struct PxControllerCollisionFlag: u8 {
        const CollisionSides = eCOLLISION_SIDES as u8;
        const CollisionUp = eCOLLISION_UP as u8;
        const CollisionDown = eCOLLISION_DOWN as u8;
    }
}

#[derive(Clone, Copy)]
pub struct PxControllerRef(*mut physx_sys::PxController);
impl PxControllerRef {
    pub fn move_controller(
        &self,
        displacement: glam::Vec3,
        min_dist: f32,
        elapsed_time: f32,
        filters: &PxControllerFilters,
        obstacles: Option<&PxObstacleContext>,
    ) -> PxControllerCollisionFlag {
        let res = unsafe {
            physx_sys::PxController_move_mut(
                self.0,
                &to_physx_vec3(displacement),
                min_dist,
                elapsed_time,
                &filters.0,
                obstacles.map(|x| x.0).unwrap_or(null_mut()),
            )
        };
        PxControllerCollisionFlag::from_bits(res.mBits).unwrap()
    }
    pub fn get_position(&self) -> glam::f64::DVec3 {
        to_glam_vec3_f64(unsafe { &*physx_sys::PxController_getPosition(self.0) })
    }
    pub fn set_position(&self, pos: glam::f64::DVec3) {
        let pos = to_physx_vec3_f64(pos);
        unsafe { physx_sys::PxController_setPosition_mut(self.0, &pos) };
    }
    pub fn get_foot_position(&self) -> glam::f64::DVec3 {
        to_glam_vec3_f64(unsafe { &physx_sys::PxController_getFootPosition(self.0) })
    }
    pub fn set_foot_position(&self, pos: glam::f64::DVec3) {
        let pos = to_physx_vec3_f64(pos);
        unsafe { physx_sys::PxController_setFootPosition_mut(self.0, &pos) };
    }
    pub fn get_actor(&self) -> PxRigidDynamicRef {
        unsafe { PxRigidDynamicRef(physx_sys::PxController_getActor(self.0)) }
    }
    pub fn release(self) {
        unsafe { physx_sys::PxController_release_mut(self.0) }
    }
}
unsafe impl Sync for PxControllerRef {}
unsafe impl Send for PxControllerRef {}
