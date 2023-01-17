use std::{
    ffi::{c_void, CStr, CString}, ptr::null_mut
};

use glam::Vec3;

use crate::{
    to_glam_vec3, to_physx_vec3, AsPxBase, PxBaseRef, PxConstraintRef, PxGeometry, PxMaterial, PxPhysicsRef, PxSceneRef, PxShape, PxTransform, PxUserData
};

pub trait AsPxActor: Sync + Send + AsPxBase {
    fn as_actor(&self) -> PxActorRef;
}
pub trait AsPxRigidActor: Sync + Send + AsPxActor {
    fn as_rigid_actor(&self) -> PxRigidActorRef;
}
pub trait AsPxRigidBody: Sync + Send + AsPxRigidActor {
    fn as_rigid_body(&self) -> PxRigidBodyRef;
}

bitflags! {
    pub struct PxActorFlag: u32 {
        const DISABLE_GRAVITY = physx_sys::PxActorFlag::eDISABLE_GRAVITY;
        const DISABLE_SIMULATION = physx_sys::PxActorFlag::eDISABLE_SIMULATION;
        const SEND_SLEEP_NOTIFIES = physx_sys::PxActorFlag::eSEND_SLEEP_NOTIFIES;
        const VISUALIZATION = physx_sys::PxActorFlag::eVISUALIZATION;
    }
}

pub trait PxActor {
    fn get_scene(&self) -> Option<PxSceneRef>;
    /// Returns an axis aligned bounding box (min, max)
    fn get_world_bounds(&self, inflation: f32) -> (Vec3, Vec3);
    fn set_actor_flag(&self, flag: PxActorFlag, value: bool);
    fn set_actor_flags(&self, flags: PxActorFlag);
    fn get_name(&self) -> String;
    fn set_name(&self, name: &CString);
}
impl<T: AsPxActor + 'static> PxActor for T {
    fn get_scene(&self) -> Option<PxSceneRef> {
        unsafe {
            let p = physx_sys::PxActor_getScene(self.as_actor().0);
            if p.is_null() {
                None
            } else {
                Some(PxSceneRef(p))
            }
        }
    }
    fn set_actor_flag(&self, flag: PxActorFlag, value: bool) {
        unsafe { physx_sys::PxActor_setActorFlag_mut(self.as_actor().0, flag.bits, value) }
    }
    fn set_actor_flags(&self, flags: PxActorFlag) {
        unsafe { physx_sys::PxActor_setActorFlags_mut(self.as_actor().0, physx_sys::PxActorFlags { mBits: flags.bits as u8 }) }
    }

    fn get_world_bounds(&self, inflation: f32) -> (Vec3, Vec3) {
        let bounds = unsafe { physx_sys::PxActor_getWorldBounds(self.as_actor().0, inflation) };
        (to_glam_vec3(&bounds.minimum), to_glam_vec3(&bounds.maximum))
    }
    fn get_name(&self) -> String {
        unsafe {
            let p = physx_sys::PxActor_getName(self.as_actor().0);
            CStr::from_ptr(p).to_str().unwrap().to_string()
        }
    }
    /// Note; physx doesn't copy the string, so the string needs to be kept alive somewhere else
    fn set_name(&self, name: &CString) {
        unsafe { physx_sys::PxActor_setName_mut(self.as_actor().0, name.as_ptr()) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PxActorRef(pub *mut physx_sys::PxActor);
impl PxActorRef {
    pub fn release(self) {
        unsafe { physx_sys::PxActor_release_mut(self.as_actor().0) }
    }
}
impl AsPxBase for PxActorRef {
    fn as_base(&self) -> PxBaseRef {
        PxBaseRef(self.0 as _)
    }
}
impl AsPxActor for PxActorRef {
    fn as_actor(&self) -> PxActorRef {
        PxActorRef(self.0 as _)
    }
}
impl PxUserData for PxActorRef {
    fn raw_user_data_mut(&self) -> &mut *mut c_void {
        unsafe { &mut (*self.0).userData }
    }
    fn raw_user_data(&self) -> &*mut c_void {
        unsafe { &(*self.0).userData }
    }
}
unsafe impl Sync for PxActorRef {}
unsafe impl Send for PxActorRef {}

pub trait PxRigidActor {
    fn attach_shape(&self, shape: &PxShape) -> bool;
    fn detach_shape(&self, shape: &PxShape, wake_on_lost_touch: bool);
    fn get_nb_shapes(&self) -> u32;
    fn get_shapes(&self) -> Vec<PxShape>;
    fn set_global_pose(&self, pose: &PxTransform, autowake: bool);
    fn get_global_pose(&self) -> PxTransform;
    fn get_constraints(&self) -> Vec<PxConstraintRef>;
    fn release(self);
}
impl<T: AsPxRigidActor + 'static> PxRigidActor for T {
    fn attach_shape(&self, shape: &PxShape) -> bool {
        unsafe { physx_sys::PxRigidActor_attachShape_mut(self.as_rigid_actor().0, shape.0) }
    }
    fn detach_shape(&self, shape: &PxShape, wake_on_lost_touch: bool) {
        unsafe { physx_sys::PxRigidActor_detachShape_mut(self.as_rigid_actor().0, shape.0, wake_on_lost_touch) }
    }
    fn get_nb_shapes(&self) -> u32 {
        unsafe { physx_sys::PxRigidActor_getNbShapes(self.as_rigid_actor().0) }
    }
    fn get_shapes(&self) -> Vec<PxShape> {
        let capacity = self.get_nb_shapes();
        let mut buffer: Vec<*mut physx_sys::PxShape> = Vec::with_capacity(capacity as usize);
        unsafe {
            let len = physx_sys::PxRigidActor_getShapes(self.as_rigid_actor().0, buffer.as_mut_ptr() as *mut *mut _, capacity, 0);
            buffer.set_len(len as usize);
        }
        buffer
            .into_iter()
            .map(|x| {
                let mut s = PxShape(x);
                s.acquire_reference();
                s
            })
            .collect()
    }
    fn set_global_pose(&self, pose: &PxTransform, autowake: bool) {
        unsafe {
            physx_sys::PxRigidActor_setGlobalPose_mut(self.as_rigid_actor().0, &pose.0, autowake);
        }
    }
    fn get_global_pose(&self) -> PxTransform {
        PxTransform(unsafe { physx_sys::PxRigidActor_getGlobalPose(self.as_rigid_actor().0) })
    }
    fn get_constraints(&self) -> Vec<PxConstraintRef> {
        unsafe {
            let count = physx_sys::PxRigidActor_getNbConstraints(self.as_rigid_actor().0);
            let mut buff = vec![null_mut(); count as usize];
            physx_sys::PxRigidActor_getConstraints(self.as_rigid_actor().0, buff.as_mut_ptr() as *mut *mut _, count, 0);
            buff.into_iter().map(PxConstraintRef).collect()
        }
    }
    fn release(self) {
        if self.as_actor().has_user_data() {
            panic!("Remove user data before releasing to avoid memory leak");
        }
        unsafe { physx_sys::PxRigidActor_release_mut(self.as_rigid_actor().0) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PxRigidActorRef(pub *mut physx_sys::PxRigidActor);
impl PxRigidActorRef {
    pub(crate) fn from_ptr(ptr: *mut physx_sys::PxRigidActor) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self(ptr))
        }
    }
}
impl AsPxBase for PxRigidActorRef {
    fn as_base(&self) -> PxBaseRef {
        PxBaseRef(self.0 as _)
    }
}
impl AsPxActor for PxRigidActorRef {
    fn as_actor(&self) -> PxActorRef {
        PxActorRef(self.0 as _)
    }
}
impl AsPxRigidActor for PxRigidActorRef {
    fn as_rigid_actor(&self) -> PxRigidActorRef {
        PxRigidActorRef(self.0 as _)
    }
}
unsafe impl Sync for PxRigidActorRef {}
unsafe impl Send for PxRigidActorRef {}

bitflags! {
    pub struct PxRigidBodyFlag: u32 {
        const ENABLE_CCD = physx_sys::PxRigidBodyFlag::eENABLE_CCD;
        const ENABLE_CCD_FRICTION = physx_sys::PxRigidBodyFlag::eENABLE_CCD_FRICTION;
        const ENABLE_CCD_MAX_CONTACT_IMPULSE = physx_sys::PxRigidBodyFlag::eENABLE_CCD_MAX_CONTACT_IMPULSE;
        const ENABLE_POSE_INTEGRATION_PREVIEW = physx_sys::PxRigidBodyFlag::eENABLE_POSE_INTEGRATION_PREVIEW;
        const ENABLE_SPECULATIVE_CCD = physx_sys::PxRigidBodyFlag::eENABLE_SPECULATIVE_CCD;
        const KINEMATIC = physx_sys::PxRigidBodyFlag::eKINEMATIC;
        const RETAIN_ACCELERATIONS = physx_sys::PxRigidBodyFlag::eRETAIN_ACCELERATIONS;
        const USE_KINEMATIC_TARGET_FOR_SCENE_QUERIES = physx_sys::PxRigidBodyFlag::eUSE_KINEMATIC_TARGET_FOR_SCENE_QUERIES;
    }
}

#[repr(u32)]
pub enum PxForceMode {
    Acceleration = physx_sys::PxForceMode::eACCELERATION,
    Force = physx_sys::PxForceMode::eFORCE,
    Impulse = physx_sys::PxForceMode::eIMPULSE,
    VelocityChange = physx_sys::PxForceMode::eVELOCITY_CHANGE,
}

pub trait PxRigidBody {
    fn update_mass_and_inertia(
        &self,
        shape_densities: Vec<f32>,
        mass_local_pose: Option<glam::Vec3>,
        include_non_sim_shapes: Option<bool>,
    ) -> bool;
    fn update_mass_and_inertia_uniform(
        &self,
        density: f32,
        mass_local_pose: Option<glam::Vec3>,
        include_non_sim_shapes: Option<bool>,
    ) -> bool;
    fn get_linear_velocity(&self) -> Vec3;
    fn set_linear_velocity(&self, value: Vec3, autoawake: bool);
    fn get_angular_velocity(&self) -> Vec3;
    fn set_angular_velocity(&self, value: Vec3, autoawake: bool);
    fn get_mass(&self) -> f32;
    fn set_mass(&self, mass: f32);
    fn get_velocity_at_pos(&self, pos: Vec3) -> Vec3;
    fn add_force(&self, force: Vec3, mode: Option<PxForceMode>, autowake: Option<bool>);
    fn add_force_at_pos(&self, force: Vec3, pos: Vec3, mode: Option<PxForceMode>, wakeup: Option<bool>);
    fn get_rigid_body_flags(&self) -> PxRigidBodyFlag;
    fn set_rigid_body_flag(&self, flag: PxRigidBodyFlag, value: bool);
    fn set_rigid_body_flags(&self, flags: PxRigidBodyFlag);
}
impl<T: AsPxRigidBody + ?Sized> PxRigidBody for T {
    fn update_mass_and_inertia(
        &self,
        shape_densities: Vec<f32>,
        mass_local_pose: Option<glam::Vec3>,
        include_non_sim_shapes: Option<bool>,
    ) -> bool {
        unsafe {
            physx_sys::PxRigidBodyExt_updateMassAndInertia_mut(
                self.as_rigid_body().0,
                shape_densities.as_ptr(),
                shape_densities.len() as u32,
                mass_local_pose.map(|x| &to_physx_vec3(x) as *const physx_sys::PxVec3).unwrap_or(null_mut()),
                include_non_sim_shapes.unwrap_or(false),
            )
        }
    }
    fn update_mass_and_inertia_uniform(
        &self,
        density: f32,
        mass_local_pose: Option<glam::Vec3>,
        include_non_sim_shapes: Option<bool>,
    ) -> bool {
        unsafe {
            physx_sys::PxRigidBodyExt_updateMassAndInertia_mut_1(
                self.as_rigid_body().0,
                density,
                mass_local_pose.map(|x| &to_physx_vec3(x) as *const physx_sys::PxVec3).unwrap_or(null_mut()),
                include_non_sim_shapes.unwrap_or(false),
            )
        }
    }
    fn get_linear_velocity(&self) -> Vec3 {
        to_glam_vec3(&unsafe { physx_sys::PxRigidBody_getLinearVelocity(self.as_rigid_body().0) })
    }
    fn set_linear_velocity(&self, value: Vec3, autoawake: bool) {
        unsafe { physx_sys::PxRigidBody_setLinearVelocity_mut(self.as_rigid_body().0, &to_physx_vec3(value), autoawake) }
    }
    fn get_angular_velocity(&self) -> Vec3 {
        to_glam_vec3(&unsafe { physx_sys::PxRigidBody_getAngularVelocity(self.as_rigid_body().0) })
    }
    fn set_angular_velocity(&self, value: Vec3, autoawake: bool) {
        unsafe { physx_sys::PxRigidBody_setAngularVelocity_mut(self.as_rigid_body().0, &to_physx_vec3(value), autoawake) }
    }
    fn get_mass(&self) -> f32 {
        unsafe { physx_sys::PxRigidBody_getMass(self.as_rigid_body().0) }
    }
    fn set_mass(&self, mass: f32) {
        unsafe { physx_sys::PxRigidBody_setMass_mut(self.as_rigid_body().0, mass) }
    }
    fn get_velocity_at_pos(&self, pos: Vec3) -> Vec3 {
        unsafe { to_glam_vec3(&physx_sys::PxRigidBodyExt_getVelocityAtPos_mut(self.as_rigid_body().0, &to_physx_vec3(pos))) }
    }
    fn add_force(&self, force: Vec3, mode: Option<PxForceMode>, wakeup: Option<bool>) {
        unsafe {
            physx_sys::PxRigidBody_addForce_mut(
                self.as_rigid_body().0,
                &to_physx_vec3(force),
                mode.unwrap_or(PxForceMode::Force) as u32,
                wakeup.unwrap_or(true),
            )
        }
    }
    fn add_force_at_pos(&self, force: Vec3, pos: Vec3, mode: Option<PxForceMode>, wakeup: Option<bool>) {
        unsafe {
            physx_sys::PxRigidBodyExt_addForceAtPos_mut(
                self.as_rigid_body().0,
                &to_physx_vec3(force),
                &to_physx_vec3(pos),
                mode.unwrap_or(PxForceMode::Force) as u32,
                wakeup.unwrap_or(true),
            )
        }
    }
    fn get_rigid_body_flags(&self) -> PxRigidBodyFlag {
        unsafe { PxRigidBodyFlag::from_bits(physx_sys::PxRigidBody_getRigidBodyFlags(self.as_rigid_body().0).mBits as u32).unwrap() }
    }
    fn set_rigid_body_flag(&self, flag: PxRigidBodyFlag, value: bool) {
        unsafe { physx_sys::PxRigidBody_setRigidBodyFlag_mut(self.as_rigid_body().0, flag.bits, value) }
    }
    fn set_rigid_body_flags(&self, flags: PxRigidBodyFlag) {
        unsafe {
            physx_sys::PxRigidBody_setRigidBodyFlags_mut(self.as_rigid_body().0, physx_sys::PxRigidBodyFlags { mBits: flags.bits as u8 })
        }
    }
}

#[derive(Clone, Copy)]
pub struct PxRigidBodyRef(pub *mut physx_sys::PxRigidBody);
impl AsPxBase for PxRigidBodyRef {
    fn as_base(&self) -> PxBaseRef {
        PxBaseRef(self.0 as _)
    }
}
impl AsPxActor for PxRigidBodyRef {
    fn as_actor(&self) -> PxActorRef {
        PxActorRef(self.0 as _)
    }
}
impl AsPxRigidActor for PxRigidBodyRef {
    fn as_rigid_actor(&self) -> PxRigidActorRef {
        PxRigidActorRef(self.0 as _)
    }
}
impl AsPxRigidBody for PxRigidBodyRef {
    fn as_rigid_body(&self) -> PxRigidBodyRef {
        PxRigidBodyRef(self.0 as _)
    }
}
unsafe impl Sync for PxRigidBodyRef {}
unsafe impl Send for PxRigidBodyRef {}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct PxRigidDynamicRef(pub *mut physx_sys::PxRigidDynamic);
impl PxRigidDynamicRef {
    pub fn new(physics: PxPhysicsRef, pose: &PxTransform) -> Self {
        Self(unsafe { physx_sys::PxPhysics_createRigidDynamic_mut(physics.0, &pose.0) })
    }
    pub fn new_with_geometry(
        physics: &PxPhysicsRef,
        transform: &PxTransform,
        geometry: &dyn PxGeometry,
        material: &PxMaterial,
        density: f32,
        shape_offset: &PxTransform,
    ) -> Self {
        Self(unsafe {
            physx_sys::phys_PxCreateDynamic(physics.0, &transform.0, geometry.as_geometry_ptr(), material.0, density, &shape_offset.0)
        })
    }
}
impl PxRigidDynamicRef {
    pub fn wake_up(&self) {
        unsafe { physx_sys::PxRigidDynamic_wakeUp_mut(self.0) }
    }
}
impl AsPxBase for PxRigidDynamicRef {
    fn as_base(&self) -> PxBaseRef {
        PxBaseRef(self.0 as _)
    }
}
impl AsPxActor for PxRigidDynamicRef {
    fn as_actor(&self) -> PxActorRef {
        PxActorRef(self.0 as _)
    }
}
impl AsPxRigidActor for PxRigidDynamicRef {
    fn as_rigid_actor(&self) -> PxRigidActorRef {
        PxRigidActorRef(self.0 as _)
    }
}
impl AsPxRigidBody for PxRigidDynamicRef {
    fn as_rigid_body(&self) -> PxRigidBodyRef {
        PxRigidBodyRef(self.0 as _)
    }
}
unsafe impl Sync for PxRigidDynamicRef {}
unsafe impl Send for PxRigidDynamicRef {}

#[derive(Debug, Clone, Copy)]
pub struct PxRigidStaticRef(pub *mut physx_sys::PxRigidStatic);
impl PxRigidStaticRef {
    pub fn new(physics: PxPhysicsRef, pose: &PxTransform) -> Self {
        Self(unsafe { physx_sys::PxPhysics_createRigidStatic_mut(physics.0, &pose.0) })
    }
    pub fn new_with_geometry(
        physics: PxPhysicsRef,
        transform: &PxTransform,
        geometry: &dyn PxGeometry,
        material: &PxMaterial,
        shape_transform: &PxTransform,
    ) -> Self {
        Self(unsafe { physx_sys::phys_PxCreateStatic(physics.0, &transform.0, geometry.as_geometry_ptr(), material.0, &shape_transform.0) })
    }
    pub fn new_plane(physics: PxPhysicsRef, normal: glam::Vec3, distance: f32, material: &PxMaterial) -> Self {
        Self(unsafe {
            physx_sys::phys_PxCreatePlane(physics.0, &physx_sys::PxPlane_new_1(normal.x, normal.y, normal.z, distance), material.0)
        })
    }
}
impl AsPxBase for PxRigidStaticRef {
    fn as_base(&self) -> PxBaseRef {
        PxBaseRef(self.0 as _)
    }
}
impl AsPxActor for PxRigidStaticRef {
    fn as_actor(&self) -> PxActorRef {
        PxActorRef(self.0 as _)
    }
}
impl AsPxRigidActor for PxRigidStaticRef {
    fn as_rigid_actor(&self) -> PxRigidActorRef {
        PxRigidActorRef(self.0 as _)
    }
}
unsafe impl Sync for PxRigidStaticRef {}
unsafe impl Send for PxRigidStaticRef {}
