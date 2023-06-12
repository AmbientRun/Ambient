use std::ffi::CStr;

use crate::PxFoundationRef;

#[repr(u32)]
pub enum PxPvdInstrumentationFlags {
    All = physx_sys::PxPvdInstrumentationFlag::eALL,
    Debug = physx_sys::PxPvdInstrumentationFlag::eDEBUG,
    Memory = physx_sys::PxPvdInstrumentationFlag::eMEMORY,
    Profile = physx_sys::PxPvdInstrumentationFlag::ePROFILE,
}

#[derive(Clone, Copy)]
pub struct PxPvdRef(pub *mut physx_sys::PxPvd);
impl PxPvdRef {
    pub fn new(foundation: &PxFoundationRef) -> Self {
        Self(unsafe { physx_sys::phys_PxCreatePvd(foundation.0) })
    }
    pub fn connect(&self, transport: &PxPvdTransportRef, flags: PxPvdInstrumentationFlags) -> bool {
        unsafe {
            physx_sys::PxPvd_connect_mut(
                self.0,
                transport.0,
                physx_sys::PxPvdInstrumentationFlags { mBits: flags as _ },
            )
        }
    }
    pub fn release(self) {
        unsafe {
            physx_sys::PxPvd_release_mut(self.0);
        }
    }
}
unsafe impl Sync for PxPvdRef {}
unsafe impl Send for PxPvdRef {}

#[derive(Clone, Copy)]
pub struct PxPvdTransportRef(*mut physx_sys::PxPvdTransport);
impl PxPvdTransportRef {
    pub fn new() -> Self {
        Self::new_default_socket_localhost(5425, 10)
    }
    pub fn new_default_socket_localhost(port: i32, timeout_in_ms: u32) -> Self {
        let oshost = unsafe { CStr::from_bytes_with_nul_unchecked(b"localhost\0") };
        Self(unsafe {
            physx_sys::phys_PxDefaultPvdSocketTransportCreate(
                oshost.as_ptr() as _,
                port,
                timeout_in_ms,
            )
        })
    }
    pub fn release(self) {
        unsafe { physx_sys::PxPvdTransport_release_mut(self.0) }
    }
}
unsafe impl Sync for PxPvdTransportRef {}
unsafe impl Send for PxPvdTransportRef {}

bitflags! {
    pub struct PxPvdSceneFlag: u32 {
        const TRANSMIT_CONSTRAINTS = physx_sys::PxPvdSceneFlag::eTRANSMIT_CONSTRAINTS;
        const TRANSMIT_CONTACTS = physx_sys::PxPvdSceneFlag::eTRANSMIT_CONTACTS;
        const TRANSMIT_SCENEQUERIES = physx_sys::PxPvdSceneFlag::eTRANSMIT_SCENEQUERIES;
    }
}

#[derive(Clone, Copy)]
pub struct PxPvdSceneClientRef(pub(crate) *mut physx_sys::PxPvdSceneClient);
impl PxPvdSceneClientRef {
    pub fn set_scene_pvd_flag(&mut self, flag: PxPvdSceneFlag, value: bool) {
        unsafe { physx_sys::PxPvdSceneClient_setScenePvdFlag_mut(self.0, flag.bits, value) }
    }
    pub fn set_scene_pvd_flags(&mut self, flags: PxPvdSceneFlag) {
        unsafe {
            physx_sys::PxPvdSceneClient_setScenePvdFlags_mut(
                self.0,
                physx_sys::PxPvdSceneFlags {
                    mBits: flags.bits as u8,
                },
            )
        }
    }
}
unsafe impl Sync for PxPvdSceneClientRef {}
unsafe impl Send for PxPvdSceneClientRef {}
