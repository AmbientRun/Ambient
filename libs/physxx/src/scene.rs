use std::ptr::null_mut;

use glam::Vec3;
use physx_sys::{
    create_overlap_buffer, create_raycast_buffer, create_sweep_buffer, delete_overlap_callback,
    delete_raycast_callback,
};
use serde::{Deserialize, Serialize};

use crate::{
    sweep::PxSweepHit, to_glam_vec3, to_physx_vec3, AsArticulationBase, AsPxActor, PxActorRef,
    PxAggregateRef, PxCollectionRef, PxConstraintRef, PxDefaultCpuDispatcherRef, PxGeometry,
    PxHitFlags, PxPhysicsRef, PxPvdSceneClientRef, PxRaycastHit, PxRigidActorRef, PxShape,
    PxTransform,
};

pub struct PxSceneDesc(physx_sys::PxSceneDesc);
impl PxSceneDesc {
    pub fn new(physics: PxPhysicsRef) -> Self {
        Self(unsafe {
            let mut scene_desc =
                physx_sys::PxSceneDesc_new(physx_sys::PxPhysics_getTolerancesScale(physics.0));

            scene_desc.filterShader = physx_sys::get_default_simulation_filter_shader();
            scene_desc.solverType = physx_sys::PxSolverType::eTGS;

            scene_desc
        })
    }
    pub fn set_gravity(&mut self, gravity: glam::Vec3) {
        self.0.gravity = to_physx_vec3(gravity);
    }
    pub fn get_gravity(&self) -> glam::Vec3 {
        to_glam_vec3(&self.0.gravity)
    }
    pub fn set_cpu_dispatcher(&mut self, dispatcher: &PxDefaultCpuDispatcherRef) {
        self.0.cpuDispatcher = dispatcher.0 as *mut physx_sys::PxCpuDispatcher;
    }
    pub fn set_filter_shader(
        &mut self,
        shader: physx_sys::SimulationFilterShader,
        call_default_filter_shader_first: bool,
    ) {
        unsafe {
            physx_sys::enable_custom_filter_shader(
                &mut self.0,
                shader,
                call_default_filter_shader_first as u32,
            );
        }
    }
    pub fn set_simulation_event_callbacks<C: FnMut(&PxContactPairHeader)>(
        &mut self,
        callbacks: PxSimulationEventCallback<C>,
    ) {
        unsafe {
            unsafe extern "C" fn collision_callback_trampoline<C: FnMut(&PxContactPairHeader)>(
                user_data: *mut std::ffi::c_void,
                pair_header: *const physx_sys::PxContactPairHeader,
                _pairs: *const physx_sys::PxContactPair,
                _nb_pairs: u32,
            ) {
                let mut cb: Box<C> = Box::from_raw(user_data as _);
                let pair_header_flags =
                    PxContactPairHeaderFlag::from_bits((*pair_header).flags.mBits).unwrap();
                cb(&PxContactPairHeader {
                    actors: [
                        if pair_header_flags.contains(PxContactPairHeaderFlag::REMOVED_ACTOR_0) {
                            None
                        } else {
                            PxRigidActorRef::from_ptr((*pair_header).actors[0])
                        },
                        if pair_header_flags.contains(PxContactPairHeaderFlag::REMOVED_ACTOR_1) {
                            None
                        } else {
                            PxRigidActorRef::from_ptr((*pair_header).actors[1])
                        },
                    ],
                });
                Box::into_raw(cb);
            }
            let mut cbs = physx_sys::SimulationEventCallbackInfo {
                ..Default::default()
            };
            if let Some(cb) = callbacks.collision_callback {
                cbs.collision_callback = Some(collision_callback_trampoline::<C>);
                cbs.collision_user_data = Box::into_raw(cb) as _;
            }
            self.0.simulationEventCallback = physx_sys::create_simulation_event_callbacks(&cbs);
        }
    }
    pub fn get_flags(&mut self) -> PxSceneFlags {
        PxSceneFlags::from_bits(self.0.flags.mBits).unwrap()
    }
    pub fn set_flags(&mut self, flags: PxSceneFlags) {
        self.0.flags = physx_sys::PxSceneFlags { mBits: flags.bits };
    }
    pub fn update_flags(&mut self, update: impl Fn(PxSceneFlags) -> PxSceneFlags) {
        let flags = self.get_flags();
        let flags = update(flags);
        self.set_flags(flags);
    }
}

pub struct PxContactPairHeader {
    pub actors: [Option<PxRigidActorRef>; 2],
}

pub struct PxSimulationEventCallback<C: FnMut(&PxContactPairHeader)> {
    pub collision_callback: Option<Box<C>>,
}

bitflags! {
    pub struct PxContactPairHeaderFlag: u16 {
        const REMOVED_ACTOR_0 = physx_sys::PxContactPairHeaderFlag::eREMOVED_ACTOR_0 as u16;
        const REMOVED_ACTOR_1 = physx_sys::PxContactPairHeaderFlag::eREMOVED_ACTOR_1 as u16;
    }
}

bitflags! {
    pub struct PxSceneFlags: u32 {
        const ADAPTIVE_FORCE = physx_sys::PxSceneFlag::eADAPTIVE_FORCE;
        const DISABLE_CCD_RESWEEP = physx_sys::PxSceneFlag::eDISABLE_CCD_RESWEEP;
        const DISABLE_CONTACT_CACHE = physx_sys::PxSceneFlag::eDISABLE_CONTACT_CACHE;
        const DISABLE_CONTACT_REPORT_BUFFER_RESIZE = physx_sys::PxSceneFlag::eDISABLE_CONTACT_REPORT_BUFFER_RESIZE;
        const ENABLE_ACTIVE_ACTORS = physx_sys::PxSceneFlag::eENABLE_ACTIVE_ACTORS;
        const ENABLE_AVERAGE_POINT = physx_sys::PxSceneFlag::eENABLE_AVERAGE_POINT;
        const ENABLE_CCD = physx_sys::PxSceneFlag::eENABLE_CCD;
        const ENABLE_ENHANCED_DETERMINISM = physx_sys::PxSceneFlag::eENABLE_ENHANCED_DETERMINISM;
        const ENABLE_FRICTION_EVERY_ITERATION = physx_sys::PxSceneFlag::eENABLE_FRICTION_EVERY_ITERATION;
        const ENABLE_GPU_DYNAMICS = physx_sys::PxSceneFlag::eENABLE_GPU_DYNAMICS;
        const ENABLE_PCM = physx_sys::PxSceneFlag::eENABLE_PCM;
        const ENABLE_STABILIZATION = physx_sys::PxSceneFlag::eENABLE_STABILIZATION;
        const EXCLUDE_KINEMATICS_FROM_ACTIVE_ACTORS = physx_sys::PxSceneFlag::eEXCLUDE_KINEMATICS_FROM_ACTIVE_ACTORS;
        const MUTABLE_FLAGS = physx_sys::PxSceneFlag::eMUTABLE_FLAGS;
        const REQUIRE_RW_LOCK = physx_sys::PxSceneFlag::eREQUIRE_RW_LOCK;
    }
}

bitflags! {
    pub struct PxVisualizationParameter: u32 {
        const ACTOR_AXES = physx_sys::PxVisualizationParameter::eACTOR_AXES;
        const BODY_ANG_VELOCITY = physx_sys::PxVisualizationParameter::eBODY_ANG_VELOCITY;
        const BODY_AXES = physx_sys::PxVisualizationParameter::eBODY_AXES;
        const BODY_LIN_VELOCITY = physx_sys::PxVisualizationParameter::eBODY_LIN_VELOCITY;
        const BODY_MASS_AXES = physx_sys::PxVisualizationParameter::eBODY_MASS_AXES;
        const COLLISION_AABBS = physx_sys::PxVisualizationParameter::eCOLLISION_AABBS;
        const COLLISION_AXES = physx_sys::PxVisualizationParameter::eCOLLISION_AXES;
        const COLLISION_COMPOUNDS = physx_sys::PxVisualizationParameter::eCOLLISION_COMPOUNDS;
        const COLLISION_DYNAMIC = physx_sys::PxVisualizationParameter::eCOLLISION_DYNAMIC;
        const COLLISION_EDGES = physx_sys::PxVisualizationParameter::eCOLLISION_EDGES;
        const COLLISION_FNORMALS = physx_sys::PxVisualizationParameter::eCOLLISION_FNORMALS;
        const COLLISION_SHAPES = physx_sys::PxVisualizationParameter::eCOLLISION_SHAPES;
        const COLLISION_STATIC = physx_sys::PxVisualizationParameter::eCOLLISION_STATIC;
        const CONTACT_ERROR = physx_sys::PxVisualizationParameter::eCONTACT_ERROR;
        const CONTACT_FORCE = physx_sys::PxVisualizationParameter::eCONTACT_FORCE;
        const CONTACT_NORMAL = physx_sys::PxVisualizationParameter::eCONTACT_NORMAL;
        const CONTACT_POINT = physx_sys::PxVisualizationParameter::eCONTACT_POINT;
        const CULL_BOX = physx_sys::PxVisualizationParameter::eCULL_BOX;
        const DEPRECATED_COLLISION_PAIRS = physx_sys::PxVisualizationParameter::eDEPRECATED_COLLISION_PAIRS;
        const FORCE_DWORD = physx_sys::PxVisualizationParameter::eFORCE_DWORD;
        const JOINT_LIMITS = physx_sys::PxVisualizationParameter::eJOINT_LIMITS;
        const JOINT_LOCAL_FRAMES = physx_sys::PxVisualizationParameter::eJOINT_LOCAL_FRAMES;
        const MBP_REGIONS = physx_sys::PxVisualizationParameter::eMBP_REGIONS;
        const NUM_VALUES = physx_sys::PxVisualizationParameter::eNUM_VALUES;
        const SCALE = physx_sys::PxVisualizationParameter::eSCALE;
        const WORLD_AXES = physx_sys::PxVisualizationParameter::eWORLD_AXES;
    }
}

bitflags! {
    pub struct PxHitFlag: u32 {
        const ASSUME_NO_INITIAL_OVERLAP = physx_sys::PxHitFlag::eASSUME_NO_INITIAL_OVERLAP;
        const DEFAULT = physx_sys::PxHitFlag::eDEFAULT;
        const FACE_INDEX = physx_sys::PxHitFlag::eFACE_INDEX;
        const MESH_ANY = physx_sys::PxHitFlag::eMESH_ANY;
        const MESH_BOTH_SIDES = physx_sys::PxHitFlag::eMESH_BOTH_SIDES;
        const MESH_MULTIPLE = physx_sys::PxHitFlag::eMESH_MULTIPLE;
        const MODIFIABLE_FLAGS = physx_sys::PxHitFlag::eMODIFIABLE_FLAGS;
        const MTD = physx_sys::PxHitFlag::eMTD;
        const NORMAL = physx_sys::PxHitFlag::eNORMAL;
        const POSITION = physx_sys::PxHitFlag::ePOSITION;
        const PRECISE_SWEEP = physx_sys::PxHitFlag::ePRECISE_SWEEP;
        const UV = physx_sys::PxHitFlag::eUV;
    }
}

bitflags! {
    pub struct PxActorTypeFlag: u32 {
        const RIGID_DYNAMIC = physx_sys::PxActorTypeFlag::eRIGID_DYNAMIC;
        const RIGID_STATIC = physx_sys::PxActorTypeFlag::eRIGID_STATIC;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PxSceneRef(pub(crate) *mut physx_sys::PxScene);
impl PxSceneRef {
    pub fn new(physics: &PxPhysicsRef, scene_desc: &PxSceneDesc) -> Self {
        Self(unsafe { physx_sys::PxPhysics_createScene_mut(physics.0, &scene_desc.0) })
    }

    pub fn add_actor(&self, actor: &dyn AsPxActor) {
        unsafe {
            physx_sys::PxScene_addActor_mut(self.0, actor.as_actor().0, null_mut());
        }
    }
    pub fn remove_actor(&self, actor: &dyn AsPxActor, wake_on_lost_touch: bool) {
        unsafe {
            physx_sys::PxScene_removeActor_mut(self.0, actor.as_actor().0, wake_on_lost_touch);
        }
    }

    pub fn add_aggregate(&self, aggregate: &PxAggregateRef) {
        unsafe {
            physx_sys::PxScene_addAggregate_mut(self.0, aggregate.0);
        }
    }
    pub fn remove_aggregate(&self, aggregate: &PxAggregateRef, wake_on_lost_touch: bool) {
        unsafe {
            physx_sys::PxScene_removeAggregate_mut(self.0, aggregate.0, wake_on_lost_touch);
        }
    }

    pub fn add_articulation(&self, articulation: &dyn AsArticulationBase) {
        unsafe {
            physx_sys::PxScene_addArticulation_mut(self.0, articulation.as_articulation_base_ptr());
        }
    }
    pub fn remove_articulation(
        &self,
        articulation: &dyn AsArticulationBase,
        wake_on_lost_touch: bool,
    ) {
        unsafe {
            physx_sys::PxScene_removeArticulation_mut(
                self.0,
                articulation.as_articulation_base_ptr(),
                wake_on_lost_touch,
            );
        }
    }

    pub fn add_collection(&self, collection: &PxCollectionRef) {
        unsafe {
            physx_sys::PxScene_addCollection_mut(self.0, collection.0);
        }
    }

    pub fn get_actors(&self, types: PxActorTypeFlag) -> Vec<PxActorRef> {
        unsafe {
            let types = physx_sys::PxActorTypeFlags {
                mBits: types.bits as u16,
            };
            let count = physx_sys::PxScene_getNbActors(self.0, types);
            let mut buffer: Vec<*mut physx_sys::PxActor> = Vec::with_capacity(count as usize);
            physx_sys::PxScene_getActors(self.0, types, buffer.as_mut_ptr() as _, count, 0);
            buffer.set_len(count as usize);
            buffer.into_iter().map(PxActorRef).collect()
        }
    }

    pub fn get_constraints(&self) -> Vec<PxConstraintRef> {
        unsafe {
            let count = physx_sys::PxScene_getNbConstraints(self.0);
            let mut buffer: Vec<*mut physx_sys::PxConstraint> = Vec::with_capacity(count as usize);
            physx_sys::PxScene_getConstraints(self.0, buffer.as_mut_ptr() as _, count, 0);
            buffer.set_len(count as usize);
            buffer.into_iter().map(PxConstraintRef).collect()
        }
    }

    pub fn get_visualization_parameter(&self, param: PxVisualizationParameter) -> f32 {
        unsafe { physx_sys::PxScene_getVisualizationParameter(self.0, param.bits) }
    }
    pub fn set_visualization_parameter(&self, param: PxVisualizationParameter, value: f32) -> bool {
        unsafe { physx_sys::PxScene_setVisualizationParameter_mut(self.0, param.bits, value) }
    }
    pub fn simulate(&self, dtime: f32) {
        unsafe {
            physx_sys::PxScene_simulate_mut(self.0, dtime, null_mut(), null_mut(), 0, true);
        }
    }
    pub fn fetch_results(&self, block: bool) -> bool {
        let mut error: u32 = 0;
        let fetched = unsafe { physx_sys::PxScene_fetchResults_mut(self.0, block, &mut error) };
        assert!(error == 0, "fetchResults has failed");
        fetched
    }
    pub fn get_scene_pvd_client(&self) -> PxPvdSceneClientRef {
        PxPvdSceneClientRef(unsafe { physx_sys::PxScene_getScenePvdClient_mut(self.0) })
    }
    pub fn get_gravity(&self) -> Vec3 {
        to_glam_vec3(&unsafe { physx_sys::PxScene_getGravity(self.0) })
    }
    pub fn set_gravity(&self, gravity: Vec3) {
        unsafe { physx_sys::PxScene_setGravity_mut(self.0, &to_physx_vec3(gravity)) }
    }
    pub fn raycast(
        &self,
        origin: Vec3,
        unit_dir: Vec3,
        distance: f32,
        hit_call: &mut PxRaycastCallback,
        hit_flags: Option<PxHitFlag>,
        filter_data: &PxQueryFilterData,
    ) -> bool {
        unsafe {
            physx_sys::PxScene_raycast(
                self.0,
                &to_physx_vec3(origin),
                &to_physx_vec3(unit_dir),
                distance,
                hit_call.0,
                physx_sys::PxHitFlags {
                    mBits: hit_flags.unwrap_or(PxHitFlag::DEFAULT).bits as u16,
                },
                &filter_data.0,
                null_mut(),
                null_mut(),
            )
        }
    }

    pub fn sweep(
        &self,
        geom: &dyn PxGeometry,
        pose: &PxTransform,
        dir: Vec3,
        max_dist: f32,
        filter: PxQueryFilterData,
    ) -> PxSweepCallback {
        let hit = PxSweepCallback::new(100);

        unsafe {
            physx_sys::PxScene_sweep(
                self.0,
                geom.as_geometry_ptr(),
                &pose.0 as *const _,
                &to_physx_vec3(dir),
                max_dist,
                hit.0,
                physx_sys::PxHitFlags {
                    mBits: (PxHitFlags::POSITION | PxHitFlags::DEFAULT).bits() as u16,
                },
                &filter.0,
                null_mut(),
                null_mut(),
                0.0,
            );
        }

        hit
    }
    pub fn overlap(
        &self,
        geometry: &dyn PxGeometry,
        pose: PxTransform,
        hit_call: &mut PxOverlapCallback,
        filter_data: &PxQueryFilterData,
    ) -> bool {
        unsafe {
            physx_sys::PxScene_overlap(
                self.0,
                geometry.as_geometry_ptr(),
                &pose.0,
                hit_call.0,
                &filter_data.0,
                null_mut(),
            )
        }
    }
    pub fn get_render_buffer(&self) -> PxRenderBuffer {
        unsafe {
            let buf = physx_sys::PxScene_getRenderBuffer_mut(self.0);

            let points = std::slice::from_raw_parts::<physx_sys::PxDebugPoint>(
                physx_sys::PxRenderBuffer_getPoints(buf),
                physx_sys::PxRenderBuffer_getNbPoints(buf) as usize,
            );
            let lines = std::slice::from_raw_parts::<physx_sys::PxDebugLine>(
                physx_sys::PxRenderBuffer_getLines(buf),
                physx_sys::PxRenderBuffer_getNbLines(buf) as usize,
            );
            PxRenderBuffer {
                points: points
                    .iter()
                    .map(|p| PxDebugPoint {
                        pos: to_glam_vec3(&p.pos),
                        color: p.color,
                    })
                    .collect(),
                lines: lines
                    .iter()
                    .map(|p| PxDebugLine {
                        pos0: to_glam_vec3(&p.pos0),
                        color0: p.color0,
                        pos1: to_glam_vec3(&p.pos1),
                        color1: p.color1,
                    })
                    .collect(),
            }
        }
    }
    pub fn release(self) {
        unsafe { physx_sys::PxScene_release_mut(self.0) }
    }
}

unsafe impl Sync for PxSceneRef {}
unsafe impl Send for PxSceneRef {}

pub struct PxQueryFilterData(physx_sys::PxQueryFilterData);
impl PxQueryFilterData {
    pub fn new() -> Self {
        Self(unsafe { physx_sys::PxQueryFilterData_new() })
    }
    pub fn set_flags(&mut self, flags: PxQueryFlag) {
        self.0.flags.mBits = flags.bits as u16;
    }
}
impl Default for PxQueryFilterData {
    fn default() -> Self {
        Self::new()
    }
}

bitflags! {
    pub struct PxQueryFlag: u32 {
        const ANY_HIT = physx_sys::PxQueryFlag::eANY_HIT;
        const DYNAMIC = physx_sys::PxQueryFlag::eDYNAMIC;
        const NO_BLOCK = physx_sys::PxQueryFlag::eNO_BLOCK;
        const POSTFILTER = physx_sys::PxQueryFlag::ePOSTFILTER;
        const PREFILTER = physx_sys::PxQueryFlag::ePREFILTER;
        const RESERVED = physx_sys::PxQueryFlag::eRESERVED;
        const STATIC = physx_sys::PxQueryFlag::eSTATIC;
    }
}
pub struct PxSweepCallback(*mut physx_sys::PxSweepCallback, Vec<physx_sys::PxSweepHit>);
impl PxSweepCallback {
    pub fn new(max_nb_touches: usize) -> Self {
        let mut s = unsafe {
            let p = create_sweep_buffer();
            let arr = (0..max_nb_touches).map(|_| (*p).block).collect::<Vec<_>>();
            Self(p, arr)
        };
        if max_nb_touches > 0 {
            unsafe {
                (*s.0).maxNbTouches = max_nb_touches as u32;
                (*s.0).touches = s.1.as_mut_ptr() as _;
            }
        }
        s
    }
    pub fn block(&self) -> Option<PxSweepHit> {
        unsafe {
            if (*self.0).hasBlock {
                Some((*self.0).block.into())
            } else {
                None
            }
        }
    }
    pub fn touches(&self) -> Vec<PxSweepHit> {
        self.1
            .iter()
            .take(unsafe { (*self.0).nbTouches as usize })
            .copied()
            .map(Into::into)
            .collect()
    }
}

pub struct PxRaycastCallback(
    *mut physx_sys::PxRaycastCallback,
    Vec<physx_sys::PxRaycastHit>,
);
impl PxRaycastCallback {
    pub fn new(max_nb_touches: usize) -> Self {
        let mut s = unsafe {
            let p = create_raycast_buffer();
            let arr = (0..max_nb_touches).map(|_| (*p).block).collect::<Vec<_>>();
            Self(p, arr)
        };
        if max_nb_touches > 0 {
            unsafe {
                (*s.0).maxNbTouches = max_nb_touches as u32;
                (*s.0).touches = s.1.as_mut_ptr() as _;
            }
        }
        s
    }
    pub fn block(&self) -> Option<PxRaycastHit> {
        unsafe {
            if (*self.0).hasBlock {
                Some(PxRaycastHit::from_px(&(*self.0).block))
            } else {
                None
            }
        }
    }
    pub fn touches(&self) -> Vec<PxRaycastHit> {
        self.1
            .iter()
            .take(unsafe { (*self.0).nbTouches as usize })
            .map(PxRaycastHit::from_px)
            .collect()
    }
}
impl Drop for PxRaycastCallback {
    fn drop(&mut self) {
        unsafe {
            delete_raycast_callback(self.0);
        }
    }
}

pub struct PxOverlapCallback(
    *mut physx_sys::PxOverlapCallback,
    Vec<physx_sys::PxOverlapHit>,
);
impl PxOverlapCallback {
    pub fn new(max_nb_touches: usize) -> Self {
        let mut s = unsafe {
            let p = create_overlap_buffer();
            let arr = (0..max_nb_touches).map(|_| (*p).block).collect::<Vec<_>>();
            Self(p, arr)
        };
        if max_nb_touches > 0 {
            unsafe {
                (*s.0).maxNbTouches = max_nb_touches as u32;
                (*s.0).touches = s.1.as_mut_ptr() as _;
            }
        }
        s
    }
    pub fn block(&self) -> Option<PxOverlapHit> {
        unsafe {
            if (*self.0).hasBlock {
                Some(PxOverlapHit::from_px(&(*self.0).block))
            } else {
                None
            }
        }
    }
    pub fn touches(&self) -> Vec<PxOverlapHit> {
        self.1
            .iter()
            .take(unsafe { (*self.0).nbTouches as usize })
            .map(PxOverlapHit::from_px)
            .collect()
    }
}
impl Drop for PxOverlapCallback {
    fn drop(&mut self) {
        unsafe {
            delete_overlap_callback(self.0);
        }
    }
}

#[derive(Debug, Clone)]
pub struct PxOverlapHit {
    pub actor: PxRigidActorRef,
    pub shape: PxShape,
    pub face_index: u32,
}
impl PxOverlapHit {
    pub(crate) fn from_px(hit: &physx_sys::PxOverlapHit) -> Self {
        Self {
            actor: PxRigidActorRef(hit.actor),
            shape: PxShape::from_ptr(hit.shape),
            face_index: hit.faceIndex,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct PxRenderBuffer {
    pub points: Vec<PxDebugPoint>,
    pub lines: Vec<PxDebugLine>,
}
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct PxDebugPoint {
    pub pos: Vec3,
    pub color: u32,
}
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct PxDebugLine {
    pub pos0: Vec3,
    pub color0: u32,
    pub pos1: Vec3,
    pub color1: u32,
}
