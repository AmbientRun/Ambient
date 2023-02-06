use crate::{
    articulation::PxArticulationCacheFlags, AsArticulationBase, AsArticulationJointBase, PxArticulationAxis, PxArticulationDriveType, PxArticulationFlag, PxArticulationJointType, PxArticulationLinkRef, PxArticulationMotion, PxPhysicsRef
};

#[derive(Clone, Copy)]
pub struct PxArticulationRef(*mut physx_sys::PxArticulationReducedCoordinate);
impl PxArticulationRef {
    pub fn new(physics: &PxPhysicsRef) -> Self {
        Self(unsafe { physx_sys::PxPhysics_createArticulationReducedCoordinate_mut(physics.0) })
    }
    pub fn get_articulation_links(&self) -> Vec<PxArticulationLinkRef> {
        unsafe {
            let capacity = physx_sys::PxArticulationBase_getNbLinks(self.0 as *const physx_sys::PxArticulationBase);
            let mut buffer: Vec<*mut physx_sys::PxArticulationLink> = Vec::with_capacity(capacity as usize);
            let len = physx_sys::PxArticulationBase_getLinks(
                self.0 as *const physx_sys::PxArticulationBase,
                buffer.as_mut_ptr() as *mut *mut _,
                capacity,
                0,
            );
            buffer.set_len(len as usize);
            buffer.into_iter().map(PxArticulationLinkRef).collect()
        }
    }
    pub fn set_flag(&mut self, flag: PxArticulationFlag, value: bool) {
        unsafe { physx_sys::PxArticulationReducedCoordinate_setArticulationFlag_mut(self.0, flag.into(), value) }
    }
    pub fn release(&mut self) {
        unsafe { physx_sys::PxArticulationReducedCoordinate_release_mut(self.0) }
    }
}
impl AsArticulationBase for PxArticulationRef {
    fn as_articulation_base_ptr(&self) -> *mut physx_sys::PxArticulationBase {
        self.0 as *mut physx_sys::PxArticulationBase
    }
}
unsafe impl Sync for PxArticulationRef {}
unsafe impl Send for PxArticulationRef {}

#[derive(Clone, Copy)]
pub struct PxArticulationJointRef(*mut physx_sys::PxArticulationJointReducedCoordinate);
impl PxArticulationJointRef {
    pub fn set_joint_type(&mut self, joint_type: PxArticulationJointType) {
        unsafe { physx_sys::PxArticulationJointReducedCoordinate_setJointType_mut(self.0, joint_type as u32) }
    }
    pub fn set_motion(&mut self, axis: PxArticulationAxis, motion: PxArticulationMotion) {
        unsafe { physx_sys::PxArticulationJointReducedCoordinate_setMotion_mut(self.0, axis as u32, motion as u32) }
    }
    pub fn set_limit(&self, axis: PxArticulationAxis, min: f32, max: f32) {
        unsafe {
            physx_sys::PxArticulationJointReducedCoordinate_setLimit_mut(self.0, axis as u32, min, max);
        }
    }
    pub fn set_drive(&self, axis: PxArticulationAxis, stiffness: f32, damping: f32, max_force: f32, drive_type: PxArticulationDriveType) {
        unsafe {
            physx_sys::PxArticulationJointReducedCoordinate_setDrive_mut(
                self.0,
                axis as u32,
                stiffness,
                damping,
                max_force,
                drive_type as u32,
            )
        }
    }
    pub fn get_drive_target(&self, axis: PxArticulationAxis) -> f32 {
        unsafe { physx_sys::PxArticulationJointReducedCoordinate_getDriveTarget_mut(self.0, axis as u32) }
    }
    pub fn set_drive_target(&mut self, axis: PxArticulationAxis, rot: f32) {
        unsafe { physx_sys::PxArticulationJointReducedCoordinate_setDriveTarget_mut(self.0, axis as u32, rot) };
    }
    pub fn get_drive_velocity(&self, axis: PxArticulationAxis) -> f32 {
        unsafe { physx_sys::PxArticulationJointReducedCoordinate_getDriveVelocity_mut(self.0, axis as u32) }
    }
    pub fn set_drive_velocity(&mut self, axis: PxArticulationAxis, target_velocity: f32) {
        unsafe { physx_sys::PxArticulationJointReducedCoordinate_setDriveVelocity_mut(self.0, axis as u32, target_velocity) };
    }
}
impl AsArticulationJointBase for PxArticulationJointRef {
    fn as_articulation_joint_base_ptr(&self) -> *mut physx_sys::PxArticulationJointBase {
        self.0 as *mut physx_sys::PxArticulationJointBase
    }
}
unsafe impl Sync for PxArticulationJointRef {}
unsafe impl Send for PxArticulationJointRef {}

#[derive(Clone, Copy)]
pub struct PxArticulationCacheRef {
    cache: *mut physx_sys::PxArticulationCache,
    articulation: PxArticulationRef,
    dof_starts: [u32; 64],
}
impl PxArticulationCacheRef {
    pub fn new(articulation: PxArticulationRef) -> Self {
        Self { cache: unsafe { physx_sys::PxArticulationReducedCoordinate_createCache(articulation.0) }, articulation, dof_starts: [0; 64] }
    }
    pub fn copy_internal_state_to_cache(&self, flags: PxArticulationCacheFlags) {
        unsafe {
            physx_sys::PxArticulationReducedCoordinate_copyInternalStateToCache(
                self.articulation.0,
                self.cache,
                physx_sys::PxArticulationCacheFlags { mBits: flags.bits() },
            )
        }
    }
    pub fn calc_dof_starts(&mut self) {
        self.dof_starts[0] = 0; //We know that the root link does not have a joint
        let links = self.articulation.get_articulation_links();

        for link in links.iter().skip(1) {
            let index = link.get_link_index() as usize;
            let dofs = link.get_inbound_joint_dof();

            self.dof_starts[index] = dofs;
        }

        let mut count = 0;
        for i in 1..links.len() {
            let dofs = self.dof_starts[i];
            self.dof_starts[i] = count;
            count += dofs;
        }
    }
    // pub fn get_joint_position(&self, link_index: usize) -> &[f32] {
    //     let offset = self.get_offset(link_index) as usize;
    //     let dofs = self.get_dofs(link_index) as usize;
    //     std::slice::from_raw_parts(self.cache.jointPosition,
    //     &ptr_to_slice!(jointPosition, self)[offset..offset + dofs]
    // }
    pub fn release(&mut self) {
        unsafe { physx_sys::PxArticulationReducedCoordinate_releaseCache(self.articulation.0, self.cache) }
    }
}
unsafe impl Sync for PxArticulationCacheRef {}
unsafe impl Send for PxArticulationCacheRef {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    #[test]
    fn create_destroy_articulation() {
        let foundation = PxFoundationRef::new();
        let physics = PxPhysicsRef::new(&foundation);
        let dispatcher = PxDefaultCpuDispatcherRef::new(2);
        let mut scene_desc = PxSceneDesc::new(physics);
        scene_desc.set_cpu_dispatcher(&dispatcher);
        let scene = PxSceneRef::new(&physics, &scene_desc);
        let articulation = PxArticulationRef::new(&physics);
        scene.add_articulation(&articulation);
    }
}
