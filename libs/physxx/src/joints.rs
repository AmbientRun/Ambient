use std::{ffi::c_void, ptr::null_mut};

use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::{to_glam_vec3, AsPxBase, PxBaseRef, PxPhysicsRef, PxRigidActorRef, PxTransform, PxUserData};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PxConstraintRef(pub(crate) *mut physx_sys::PxConstraint);
impl PxConstraintRef {
    pub fn get_external_reference(&self) -> PxBaseRef {
        unsafe {
            let mut type_id: u32 = 0;
            PxBaseRef(physx_sys::PxConstraint_getExternalReference_mut(self.0, &mut type_id as *mut u32) as _)
        }
    }
    pub fn get_actors(&self) -> (PxRigidActorRef, PxRigidActorRef) {
        unsafe {
            let mut actor0 = null_mut();
            let mut actor1 = null_mut();
            physx_sys::PxConstraint_getActors(self.0, &mut actor0 as _, &mut actor1 as _);
            (PxRigidActorRef(actor0), PxRigidActorRef(actor1))
        }
    }
    pub fn set_actors(&self, actor0: PxRigidActorRef, actor1: PxRigidActorRef) {
        unsafe { physx_sys::PxConstraint_setActors_mut(self.0, actor0.0, actor1.0) }
    }
    pub fn get_force(&self, linear: &mut Vec3, angular: &mut Vec3) {
        unsafe {
            let mut l = physx_sys::PxVec3_new();
            let mut a = physx_sys::PxVec3_new();
            physx_sys::PxConstraint_getForce(self.0, &mut l as *mut physx_sys::PxVec3, &mut a as *mut physx_sys::PxVec3);
            *linear = to_glam_vec3(&l);
            *angular = to_glam_vec3(&a);
        }
    }
    pub fn release(self) {
        unsafe { physx_sys::PxConstraint_release_mut(self.0) }
    }
}
impl AsPxBase for PxConstraintRef {
    fn as_base(&self) -> PxBaseRef {
        PxBaseRef(self.0 as _)
    }
}
unsafe impl Sync for PxConstraintRef {}
unsafe impl Send for PxConstraintRef {}

bitflags! {
    pub struct PxConstraintFlags: u32 {
        const BROKEN = physx_sys::PxConstraintFlag::eBROKEN;
        const COLLISION_ENABLED = physx_sys::PxConstraintFlag::eCOLLISION_ENABLED;
        const DISABLE_PREPROCESSING = physx_sys::PxConstraintFlag::eDISABLE_PREPROCESSING;
        const DRIVE_LIMITS_ARE_FORCES = physx_sys::PxConstraintFlag::eDRIVE_LIMITS_ARE_FORCES;
        const ENABLE_EXTENDED_LIMITS = physx_sys::PxConstraintFlag::eENABLE_EXTENDED_LIMITS;
        const GPU_COMPATIBLE = physx_sys::PxConstraintFlag::eGPU_COMPATIBLE;
        const IMPROVED_SLERP = physx_sys::PxConstraintFlag::eIMPROVED_SLERP;
        const PROJECTION = physx_sys::PxConstraintFlag::ePROJECTION;
        const PROJECT_TO_ACTOR0 = physx_sys::PxConstraintFlag::ePROJECT_TO_ACTOR0;
        const PROJECT_TO_ACTOR1 = physx_sys::PxConstraintFlag::ePROJECT_TO_ACTOR1;
        const VISUALIZATION = physx_sys::PxConstraintFlag::eVISUALIZATION;
    }
}

pub trait AsPxJoint: Sync + Send {
    fn as_joint(&self) -> PxJointRef;
}
pub trait PxJoint: Sync + Send {
    fn set_break_force(&self, force: f32, torque: f32);
    fn get_constraint(&self) -> PxConstraintRef;
    fn get_constraint_flags(&self) -> PxConstraintFlags;
    fn set_constraint_flags(&self, flags: PxConstraintFlags);
    fn set_constraint_flag(&self, flag: PxConstraintFlags, value: bool);
    fn get_local_pose(&self, actor: u32) -> PxTransform;
    fn set_local_pose(&self, actor: u32, local_pose: &PxTransform);
    fn get_actors(&self) -> (Option<PxRigidActorRef>, Option<PxRigidActorRef>);
    fn set_actors(&self, actor0: Option<PxRigidActorRef>, actor1: Option<PxRigidActorRef>);
    fn release(self);
}
impl<T: AsPxJoint> PxJoint for T {
    fn set_break_force(&self, force: f32, torque: f32) {
        unsafe {
            physx_sys::PxJoint_setBreakForce_mut(self.as_joint().0, force, torque);
        }
    }
    fn get_constraint(&self) -> PxConstraintRef {
        PxConstraintRef(unsafe { physx_sys::PxJoint_getConstraint(self.as_joint().0) })
    }
    fn get_constraint_flags(&self) -> PxConstraintFlags {
        PxConstraintFlags::from_bits(unsafe { physx_sys::PxJoint_getConstraintFlags(self.as_joint().0) }.mBits as u32).unwrap()
    }
    fn set_constraint_flags(&self, flags: PxConstraintFlags) {
        unsafe { physx_sys::PxJoint_setConstraintFlags_mut(self.as_joint().0, physx_sys::PxConstraintFlags { mBits: flags.bits as u16 }) }
    }
    fn set_constraint_flag(&self, flag: PxConstraintFlags, value: bool) {
        unsafe { physx_sys::PxJoint_setConstraintFlag_mut(self.as_joint().0, flag.bits, value) }
    }
    fn get_local_pose(&self, actor: u32) -> PxTransform {
        unsafe { PxTransform(physx_sys::PxJoint_getLocalPose(self.as_joint().0, actor)) }
    }
    fn set_local_pose(&self, actor: u32, local_pose: &PxTransform) {
        unsafe { physx_sys::PxJoint_setLocalPose_mut(self.as_joint().0, actor, &local_pose.0) }
    }
    fn get_actors(&self) -> (Option<PxRigidActorRef>, Option<PxRigidActorRef>) {
        unsafe {
            let mut actor0 = null_mut() as *mut physx_sys::PxRigidActor;
            let mut actor1 = null_mut() as *mut physx_sys::PxRigidActor;
            physx_sys::PxJoint_getActors(self.as_joint().0, &mut actor0 as _, &mut actor1 as _);
            let a0 = if actor0.is_null() { None } else { Some(PxRigidActorRef(actor0)) };
            let a1 = if actor1.is_null() { None } else { Some(PxRigidActorRef(actor1)) };
            (a0, a1)
        }
    }
    fn set_actors(&self, actor0: Option<PxRigidActorRef>, actor1: Option<PxRigidActorRef>) {
        unsafe {
            physx_sys::PxJoint_setActors_mut(
                self.as_joint().0,
                actor0.map(|x| x.0).unwrap_or(null_mut()),
                actor1.map(|x| x.0).unwrap_or(null_mut()),
            )
        }
    }
    fn release(self) {
        unsafe { physx_sys::PxJoint_release_mut(self.as_joint().0) }
    }
}
impl<T: AsPxJoint + 'static> PxUserData for T {
    fn raw_user_data_mut(&self) -> &mut *mut c_void {
        unsafe { &mut (*self.as_joint().0).userData }
    }
    fn raw_user_data(&self) -> &*mut c_void {
        unsafe { &(*self.as_joint().0).userData }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PxJointRef(pub *mut physx_sys::PxJoint);
impl AsPxBase for PxJointRef {
    fn as_base(&self) -> PxBaseRef {
        PxBaseRef(self.0 as _)
    }
}
impl AsPxJoint for PxJointRef {
    fn as_joint(&self) -> PxJointRef {
        PxJointRef(self.0)
    }
}
unsafe impl Sync for PxJointRef {}
unsafe impl Send for PxJointRef {}

#[derive(Debug, Clone, Copy)]
pub struct PxFixedJointRef(pub(crate) *mut physx_sys::PxFixedJoint);
impl PxFixedJointRef {
    pub fn new(
        physics: PxPhysicsRef,
        actor0: Option<PxRigidActorRef>,
        local_frame_0: &PxTransform,
        actor1: Option<PxRigidActorRef>,
        local_frame_1: &PxTransform,
    ) -> Self {
        Self(unsafe {
            physx_sys::phys_PxFixedJointCreate(
                physics.0,
                actor0.map_or(null_mut(), |v| v.0),
                &local_frame_0.0,
                actor1.map_or(null_mut(), |v| v.0),
                &local_frame_1.0,
            )
        })
    }
}
impl AsPxBase for PxFixedJointRef {
    fn as_base(&self) -> PxBaseRef {
        PxBaseRef(self.0 as _)
    }
}
impl AsPxJoint for PxFixedJointRef {
    fn as_joint(&self) -> PxJointRef {
        PxJointRef(self.0 as _)
    }
}
unsafe impl Sync for PxFixedJointRef {}
unsafe impl Send for PxFixedJointRef {}

bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct PxRevoluteJointFlag: u32 {
        const DRIVE_ENABLED = physx_sys::PxRevoluteJointFlag::eDRIVE_ENABLED;
        const DRIVE_FREESPIN = physx_sys::PxRevoluteJointFlag::eDRIVE_FREESPIN;
        const LIMIT_ENABLED = physx_sys::PxRevoluteJointFlag::eLIMIT_ENABLED;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PxRevoluteJointRef(pub(crate) *mut physx_sys::PxRevoluteJoint);
impl PxRevoluteJointRef {
    pub fn new(
        physics: PxPhysicsRef,
        actor0: Option<PxRigidActorRef>,
        local_frame_0: &PxTransform,
        actor1: Option<PxRigidActorRef>,
        local_frame_1: &PxTransform,
    ) -> Self {
        Self(unsafe {
            physx_sys::phys_PxRevoluteJointCreate(
                physics.0,
                actor0.map_or(null_mut(), |v| v.0),
                &local_frame_0.0,
                actor1.map_or(null_mut(), |v| v.0),
                &local_frame_1.0,
            )
        })
    }
    pub fn set_drive_velocity(&self, velocity: f32, autoawake: bool) {
        unsafe { physx_sys::PxRevoluteJoint_setDriveVelocity_mut(self.0, velocity, autoawake) }
    }
    pub fn get_revolute_flags(&self) -> PxRevoluteJointFlag {
        PxRevoluteJointFlag::from_bits(unsafe { physx_sys::PxRevoluteJoint_getRevoluteJointFlags(self.0) }.mBits as u32).unwrap()
    }
    pub fn set_revolute_flags(&self, flags: PxRevoluteJointFlag) {
        unsafe {
            physx_sys::PxRevoluteJoint_setRevoluteJointFlags_mut(self.0, physx_sys::PxRevoluteJointFlags { mBits: flags.bits() as _ })
        }
    }
    pub fn set_revolute_flag(&self, flag: PxRevoluteJointFlag, value: bool) {
        unsafe { physx_sys::PxRevoluteJoint_setRevoluteJointFlag_mut(self.0, flag.bits() as _, value) }
    }
    pub fn get_limit(&self) -> PxJointAngularLimitPair {
        PxJointAngularLimitPair::from_physx(unsafe { physx_sys::PxRevoluteJoint_getLimit(self.0) })
    }
    pub fn set_limit(&self, limits: &PxJointAngularLimitPair) {
        unsafe { physx_sys::PxRevoluteJoint_setLimit_mut(self.0, &limits.to_physx() as _) }
    }
}
impl AsPxBase for PxRevoluteJointRef {
    fn as_base(&self) -> PxBaseRef {
        PxBaseRef(self.0 as _)
    }
}
impl AsPxJoint for PxRevoluteJointRef {
    fn as_joint(&self) -> PxJointRef {
        PxJointRef(self.0 as _)
    }
}
unsafe impl Sync for PxRevoluteJointRef {}
unsafe impl Send for PxRevoluteJointRef {}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PxJointAngularLimitPair {
    pub restitution: f32,
    pub bounce_threshold: f32,
    pub stiffness: f32,
    pub damping: f32,
    pub contact_distance: f32,
    pub upper: f32,
    pub lower: f32,
}
impl PxJointAngularLimitPair {
    pub fn new(lower_limit: f32, upper_limit: f32, contact_dist: f32) -> Self {
        Self::from_physx(unsafe { physx_sys::PxJointAngularLimitPair_new(lower_limit, upper_limit, contact_dist) })
    }
    fn from_physx(limit: physx_sys::PxJointAngularLimitPair) -> Self {
        Self {
            restitution: limit.restitution,
            bounce_threshold: limit.bounceThreshold,
            stiffness: limit.stiffness,
            damping: limit.damping,
            contact_distance: limit.contactDistance,
            upper: limit.upper,
            lower: limit.lower,
        }
    }
    fn to_physx(&self) -> physx_sys::PxJointAngularLimitPair {
        physx_sys::PxJointAngularLimitPair {
            restitution: self.restitution,
            bounceThreshold: self.bounce_threshold,
            stiffness: self.stiffness,
            damping: self.damping,
            contactDistance: self.contact_distance,
            upper: self.upper,
            lower: self.lower,
        }
    }
}
