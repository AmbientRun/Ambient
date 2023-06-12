#![allow(non_upper_case_globals)]
#[macro_use]
extern crate bitflags;
extern crate num_derive;
use std::ptr::null_mut;

mod actor;
pub mod articulation;
pub mod articulation_reduced_coordinate;
mod base;
mod character_controller;
mod cooking;
mod geometry;
mod height_field;
mod joints;
mod mesh;
mod pvd;
mod raycast;
mod scene;
mod serialization;
mod shape;
pub mod sweep;
mod transform;
mod user_data;

pub use actor::*;
pub use articulation::*;
pub use base::*;
pub use character_controller::*;
pub use cooking::*;
pub use geometry::*;
pub use height_field::*;
pub use joints::*;
pub use mesh::*;
pub use physx_sys as sys;
pub use pvd::*;
pub use raycast::*;
pub use scene::*;
pub use serialization::*;
pub use shape::*;
pub use transform::*;
pub use user_data::*;

#[derive(Clone, Copy)]
pub struct PxFoundationRef(*mut physx_sys::PxFoundation);
impl PxFoundationRef {
    pub fn new() -> Self {
        unsafe { Self(physx_sys::physx_create_foundation()) }
    }
    pub fn get() -> Self {
        unsafe {
            Self(physx_sys::PxPhysics_getFoundation_mut(
                PxPhysicsRef::get().0,
            ))
        }
    }
    pub fn release(self) {
        unsafe { physx_sys::PxFoundation_release_mut(self.0) }
    }
}
unsafe impl Sync for PxFoundationRef {}
unsafe impl Send for PxFoundationRef {}

pub const PX_PHYSICS_VERSION: u32 = physx_sys::version(4, 1, 1);

#[derive(Clone, Copy)]
pub struct PxPhysicsRef(*mut physx_sys::PxPhysics);
impl PxPhysicsRef {
    pub fn new(foundation: &PxFoundationRef) -> Self {
        Self(unsafe {
            physx_sys::phys_PxCreatePhysics(
                PX_PHYSICS_VERSION,
                foundation.0,
                &physx_sys::PxTolerancesScale_new(),
                true,
                null_mut(),
            )
        })
    }
    pub fn new_with_pvd(foundation: &PxFoundationRef, pvd: &PxPvdRef) -> Self {
        Self(unsafe {
            physx_sys::phys_PxCreatePhysics(
                PX_PHYSICS_VERSION,
                foundation.0,
                &physx_sys::PxTolerancesScale_new(),
                true,
                pvd.0,
            )
        })
    }
    pub fn get() -> Self {
        Self(unsafe { physx_sys::phys_PxGetPhysics() })
    }
    fn get_physics_insertion_callback(&self) -> *mut physx_sys::PxPhysicsInsertionCallback {
        unsafe { physx_sys::PxPhysics_getPhysicsInsertionCallback_mut(self.0) }
    }
    fn get_tolerances_scale(&self) -> *const physx_sys::PxTolerancesScale {
        unsafe { physx_sys::PxPhysics_getTolerancesScale(self.0) }
    }
    pub fn release(self) {
        unsafe {
            physx_sys::PxPhysics_release_mut(self.0);
        }
    }
}
unsafe impl Sync for PxPhysicsRef {}
unsafe impl Send for PxPhysicsRef {}

pub fn px_init_extensions(physics: &PxPhysicsRef, pvd: &PxPvdRef) {
    assert!(unsafe { physx_sys::phys_PxInitExtensions(physics.0, pvd.0) })
}

#[derive(Clone, Copy)]
pub struct PxDefaultCpuDispatcherRef(*mut physx_sys::PxDefaultCpuDispatcher);
impl PxDefaultCpuDispatcherRef {
    pub fn new(num_threads: u32) -> Self {
        Self(unsafe { physx_sys::phys_PxDefaultCpuDispatcherCreate(num_threads, null_mut()) })
    }
    pub fn release(self) {
        unsafe { physx_sys::PxDefaultCpuDispatcher_release_mut(self.0) }
    }
}
unsafe impl Sync for PxDefaultCpuDispatcherRef {}
unsafe impl Send for PxDefaultCpuDispatcherRef {}

#[derive(Debug)]
pub struct PxMaterial(*mut physx_sys::PxMaterial);
impl PxMaterial {
    pub fn new(
        physics: PxPhysicsRef,
        static_friction: f32,
        dynamic_friction: f32,
        restitution: f32,
    ) -> Self {
        Self(unsafe {
            physx_sys::PxPhysics_createMaterial_mut(
                physics.0,
                static_friction,
                dynamic_friction,
                restitution,
            )
        })
    }
    pub(crate) fn from_ptr(ptr: *mut physx_sys::PxMaterial) -> Self {
        let mut s = Self(ptr);
        s.acquire_reference();
        s
    }
    pub(crate) fn acquire_reference(&mut self) {
        unsafe { physx_sys::PxMaterial_acquireReference_mut(self.0) }
    }
}
impl AsPxBase for PxMaterial {
    fn as_base(&self) -> PxBaseRef {
        PxBaseRef(self.0 as _)
    }
}
impl Drop for PxMaterial {
    fn drop(&mut self) {
        unsafe { physx_sys::PxMaterial_release_mut(self.0) }
    }
}
impl Clone for PxMaterial {
    fn clone(&self) -> Self {
        Self::from_ptr(self.0)
    }
}
unsafe impl Sync for PxMaterial {}
unsafe impl Send for PxMaterial {}

#[derive(Debug, Clone, Copy)]
pub struct PxAggregateRef(*mut physx_sys::PxAggregate);
impl PxAggregateRef {
    pub fn new(physics: &PxPhysicsRef, max_size: u32, self_collisions: bool) -> Self {
        Self(unsafe {
            physx_sys::PxPhysics_createAggregate_mut(physics.0, max_size, self_collisions)
        })
    }
    pub fn add_actor(&mut self, actor: &dyn AsPxActor) -> bool {
        unsafe { physx_sys::PxAggregate_addActor_mut(self.0, actor.as_actor().0, null_mut()) }
    }
    pub fn get_nb_actors(&self) -> u32 {
        unsafe { physx_sys::PxAggregate_getNbActors(self.0) }
    }
    pub fn get_max_nb_actors(&self) -> u32 {
        unsafe { physx_sys::PxAggregate_getMaxNbActors(self.0) }
    }
    pub fn release(&mut self) {
        unsafe { physx_sys::PxAggregate_release_mut(self.0) }
    }
}
impl AsPxBase for PxAggregateRef {
    fn as_base(&self) -> PxBaseRef {
        PxBaseRef(self.0 as _)
    }
}
unsafe impl Sync for PxAggregateRef {}
unsafe impl Send for PxAggregateRef {}

pub trait PxReferenceCounted {
    fn get_reference_count(&self) -> u32;
}
