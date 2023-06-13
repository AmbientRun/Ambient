use std::ptr::null_mut;

use crate::{
    AsPxActor, AsPxBase, AsPxRigidActor, AsPxRigidBody, PxActorRef, PxBaseRef, PxRigidActorRef,
    PxRigidBodyRef, PxTransform,
};

pub trait AsArticulationBase {
    fn as_articulation_base_ptr(&self) -> *mut physx_sys::PxArticulationBase;
}

pub trait AsArticulationJointBase {
    fn as_articulation_joint_base_ptr(&self) -> *mut physx_sys::PxArticulationJointBase;
}

pub trait PxArticulationBase {
    fn wake_up(&self);
    fn set_solver_iteration_counts(
        &self,
        min_position_iterations: u32,
        min_velocity_iterations: u32,
    );
    fn get_solver_iteration_counts(&self) -> (u32, u32);
}
impl<T: AsArticulationBase> PxArticulationBase for T {
    fn wake_up(&self) {
        unsafe { physx_sys::PxArticulationBase_wakeUp_mut(self.as_articulation_base_ptr()) }
    }
    fn set_solver_iteration_counts(
        &self,
        min_position_iterations: u32,
        min_velocity_iterations: u32,
    ) {
        unsafe {
            physx_sys::PxArticulationBase_setSolverIterationCounts_mut(
                self.as_articulation_base_ptr(),
                min_position_iterations,
                min_velocity_iterations,
            )
        }
    }
    fn get_solver_iteration_counts(&self) -> (u32, u32) {
        let mut p = 0u32;
        let mut v = 0u32;
        unsafe {
            physx_sys::PxArticulationBase_getSolverIterationCounts(
                self.as_articulation_base_ptr(),
                &mut p,
                &mut v,
            )
        }
        (p, v)
    }
}

pub trait PxArticulationJointBase {
    fn get_parent_pose(&self) -> PxTransform;
    fn get_child_pose(&self) -> PxTransform;
    fn set_parent_pose(&self, pose: &PxTransform);
    fn set_child_pose(&self, pose: &PxTransform);
}
impl<T: AsArticulationJointBase> PxArticulationJointBase for T {
    fn get_parent_pose(&self) -> PxTransform {
        let t = unsafe {
            physx_sys::PxArticulationJointBase_getParentPose(self.as_articulation_joint_base_ptr())
        };
        PxTransform(t)
    }
    fn get_child_pose(&self) -> PxTransform {
        let t = unsafe {
            physx_sys::PxArticulationJointBase_getChildPose(self.as_articulation_joint_base_ptr())
        };
        PxTransform(t)
    }
    fn set_parent_pose(&self, pose: &PxTransform) {
        unsafe {
            physx_sys::PxArticulationJointBase_setParentPose_mut(
                self.as_articulation_joint_base_ptr(),
                &pose.0,
            )
        }
    }
    fn set_child_pose(&self, pose: &PxTransform) {
        unsafe {
            physx_sys::PxArticulationJointBase_setChildPose_mut(
                self.as_articulation_joint_base_ptr(),
                &pose.0,
            )
        }
    }
}
pub struct PxArticulationJointBaseRef(*mut physx_sys::PxArticulationJointBase);

#[derive(Debug, Clone, Copy)]
pub struct PxArticulationLinkRef(pub(crate) *mut physx_sys::PxArticulationLink);
impl PxArticulationLinkRef {
    pub fn new(
        articulation: &dyn AsArticulationBase,
        parent: Option<&PxArticulationLinkRef>,
        pose: &PxTransform,
    ) -> Self {
        Self(unsafe {
            physx_sys::PxArticulationBase_createLink_mut(
                articulation.as_articulation_base_ptr(),
                parent.map(|x| x.0).unwrap_or(null_mut()),
                &pose.0,
            )
        })
    }
    pub fn get_inbound_joint(&self) -> PxArticulationJointBaseRef {
        PxArticulationJointBaseRef(unsafe {
            physx_sys::PxArticulationLink_getInboundJoint(self.0) as _
        })
    }
    pub fn get_link_index(&self) -> u32 {
        unsafe { physx_sys::PxArticulationLink_getLinkIndex(self.0) }
    }
    pub fn get_inbound_joint_dof(&self) -> u32 {
        unsafe { physx_sys::PxArticulationLink_getInboundJointDof(self.0) }
    }
    pub fn release(&mut self) {
        unsafe { physx_sys::PxArticulationLink_release_mut(self.0) }
    }
}
impl AsPxBase for PxArticulationLinkRef {
    fn as_base(&self) -> PxBaseRef {
        PxBaseRef(self.0 as _)
    }
}
impl AsPxActor for PxArticulationLinkRef {
    fn as_actor(&self) -> PxActorRef {
        PxActorRef(self.0 as _)
    }
}
impl AsPxRigidActor for PxArticulationLinkRef {
    fn as_rigid_actor(&self) -> PxRigidActorRef {
        PxRigidActorRef(self.0 as _)
    }
}
impl AsPxRigidBody for PxArticulationLinkRef {
    fn as_rigid_body(&self) -> PxRigidBodyRef {
        PxRigidBodyRef(self.0 as _)
    }
}
unsafe impl Sync for PxArticulationLinkRef {}
unsafe impl Send for PxArticulationLinkRef {}

bitflags! {
    pub struct PxArticulationCacheFlags: u8 {
        const Velocity     = 0b00000001;
        const Acceleration = 0b00000010;
        const Position     = 0b00000100;
        const Force        = 0b00001000;
        const Root         = 0b00010000;
        const All = Self::Velocity.bits | Self::Acceleration.bits | Self::Position.bits | Self::Force.bits | Self::Root.bits;
    }
}
impl Default for PxArticulationCacheFlags {
    fn default() -> Self {
        Self::All
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PxArticulationJointType {
    Prismatic = physx_sys::PxArticulationJointType::ePRISMATIC,
    Revolute = physx_sys::PxArticulationJointType::eREVOLUTE,
    Spherical = physx_sys::PxArticulationJointType::eSPHERICAL,
    Fix = physx_sys::PxArticulationJointType::eFIX,
    Undefined = physx_sys::PxArticulationJointType::eUNDEFINED,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum PxArticulationMotion {
    Locked = physx_sys::PxArticulationMotion::eLOCKED,
    Limited = physx_sys::PxArticulationMotion::eLIMITED,
    Free = physx_sys::PxArticulationMotion::eFREE,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum PxArticulationAxis {
    Twist = physx_sys::PxArticulationAxis::eTWIST,
    Swing1 = physx_sys::PxArticulationAxis::eSWING1,
    Swing2 = physx_sys::PxArticulationAxis::eSWING2,
    X = physx_sys::PxArticulationAxis::eX,
    Y = physx_sys::PxArticulationAxis::eY,
    Z = physx_sys::PxArticulationAxis::eZ,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum PxArticulationDriveType {
    Acceleration = physx_sys::PxArticulationDriveType::eACCELERATION,
    Force = physx_sys::PxArticulationDriveType::eFORCE,
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum PxArticulationFlag {
    FixBase = 1 << 0,
    DriveLimitsAreForces = 1 << 1,
}

impl From<PxArticulationFlag> for physx_sys::PxArticulationFlag::Enum {
    fn from(sys: PxArticulationFlag) -> Self {
        sys as _
    }
}
