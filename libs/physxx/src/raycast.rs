use glam::Vec3;

use crate::{to_glam_vec3, to_physx_vec3, PxGeometry, PxRigidActorRef, PxShape, PxTransform};

#[derive(Debug, Clone)]
pub struct PxRaycastHit {
    pub actor: Option<PxRigidActorRef>,
    pub shape: Option<PxShape>,
    pub face_index: u32,
    pub flags: PxHitFlags,
    pub position: Vec3,
    pub normal: Vec3,
    pub distance: f32,
    pub u: f32,
    pub v: f32,
}
impl PxRaycastHit {
    pub(crate) fn from_px(hit: &physx_sys::PxRaycastHit) -> Self {
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
            flags: PxHitFlags::from_bits(hit.flags.mBits as u32).unwrap(),
            position: to_glam_vec3(&hit.position),
            normal: to_glam_vec3(&hit.normal),
            distance: hit.distance,
            u: hit.u,
            v: hit.v,
        }
    }
}

bitflags! {
    pub struct PxHitFlags: u32 {
        const POSITION = physx_sys::PxHitFlag::ePOSITION;
        const NORMAL = physx_sys::PxHitFlag::eNORMAL;
        const UV = physx_sys::PxHitFlag::eUV;
        const ASSUME_NO_INITIAL_OVERLAP = physx_sys::PxHitFlag::eASSUME_NO_INITIAL_OVERLAP;
        const MESH_MULTIPLE = physx_sys::PxHitFlag::eMESH_MULTIPLE;
        const MESH_ANY = physx_sys::PxHitFlag::eMESH_ANY;
        const MESH_BOTH_SIDES = physx_sys::PxHitFlag::eMESH_BOTH_SIDES;
        const PRECISE_SWEEP = physx_sys::PxHitFlag::ePRECISE_SWEEP;
        const MTD = physx_sys::PxHitFlag::eMTD;
        const FACE_INDEX = physx_sys::PxHitFlag::eFACE_INDEX;
        const DEFAULT = physx_sys::PxHitFlag::eDEFAULT;
        const MODIFIABLE_FLAGS = physx_sys::PxHitFlag::eMODIFIABLE_FLAGS;
    }
}

pub fn raycast(
    origin: Vec3,
    unit_dir: Vec3,
    geom: &dyn PxGeometry,
    pose: &PxTransform,
    max_dist: f32,
    hit_flags: PxHitFlags,
    max_hits: u32,
) -> Vec<PxRaycastHit> {
    unsafe {
        let mut hits: Vec<_> = (0..max_hits)
            .map(|_| physx_sys::PxRaycastHit_new())
            .collect();
        let n_hits = physx_sys::PxGeometryQuery_raycast_mut(
            &to_physx_vec3(origin) as *const physx_sys::PxVec3,
            &to_physx_vec3(unit_dir) as *const physx_sys::PxVec3,
            geom.as_geometry_ptr(),
            &pose.0 as *const physx_sys::PxTransform,
            max_dist,
            physx_sys::PxHitFlags {
                mBits: hit_flags.bits as u16,
            },
            max_hits,
            hits.as_mut_ptr() as _,
        );
        hits.into_iter()
            .take(n_hits as usize)
            .map(|hit| PxRaycastHit::from_px(&hit))
            .collect()
    }
}
