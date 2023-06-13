use glam::Vec3;

use crate::{
    to_glam_vec3, to_physx_vec3, PxGeometry, PxHitFlags, PxRigidActorRef, PxShape, PxTransform,
};

#[derive(Debug, Clone)]
pub struct PxSweepHit {
    pub actor: Option<PxRigidActorRef>,
    pub shape: Option<PxShape>,
    pub face_index: u32,
    pub flags: PxHitFlags,
    pub position: Vec3,
    pub normal: Vec3,
    pub distance: f32,
}

impl From<physx_sys::PxSweepHit> for PxSweepHit {
    fn from(hit: physx_sys::PxSweepHit) -> Self {
        Self {
            actor: if !hit.actor.is_null() {
                Some(PxRigidActorRef(hit.actor))
            } else {
                None
            },
            shape: if !hit.shape.is_null() {
                Some(PxShape::from_ptr(hit.shape))
            } else {
                None
            },
            face_index: hit.faceIndex,
            flags: PxHitFlags::from_bits(hit.flags.mBits as _).expect("Invalid bits"),
            position: to_glam_vec3(&hit.position),
            normal: to_glam_vec3(&hit.normal),
            distance: hit.distance,
        }
    }
}

pub struct SweepConfig<'a> {
    pub dir: Vec3,
    pub max_dist: f32,
    pub src_pose: &'a PxTransform,
    pub src_geom: &'a dyn PxGeometry,
    pub dst_pose: &'a PxTransform,
    pub dst_geom: &'a dyn PxGeometry,
    pub flags: PxHitFlags,
    pub inflation: f32,
}

impl SweepConfig<'_> {
    /// Performs a sweep of src onto dst
    pub fn sweep(&self) -> Option<PxSweepHit> {
        assert!(self.dir.is_normalized());
        unsafe {
            let mut out = physx_sys::PxSweepHit_new();
            let hit = physx_sys::PxGeometryQuery_sweep_mut(
                &to_physx_vec3(self.dir),
                self.max_dist,
                self.src_geom.as_geometry_ptr(),
                &self.src_pose.0 as *const _,
                self.dst_geom.as_geometry_ptr(),
                &self.dst_pose.0 as *const _,
                &mut out as *mut _,
                physx_sys::PxHitFlags {
                    mBits: self.flags.bits() as _,
                },
                self.inflation,
            );
            if hit {
                Some(out.into())
            } else {
                None
            }
        }
    }
}
