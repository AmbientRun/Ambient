use crate::{
    AsPxJoint, AsPxRigidActor, AsPxRigidBody, PxAggregateRef, PxArticulationLinkRef, PxConstraintRef, PxConvexMesh, PxFixedJointRef, PxHeightField, PxJointRef, PxMaterial, PxRevoluteJointRef, PxRigidActorRef, PxRigidBodyRef, PxRigidDynamicRef, PxRigidStaticRef, PxShape
};

pub trait AsPxBase: Sync + Send {
    fn as_base(&self) -> PxBaseRef;
}
pub trait PxBase: Sync + Send + as_any::AsAny {
    fn get_concrete_type(&self) -> u16;
    fn as_px_any(&self) -> PxAny;
    fn to_rigid_dynamic(&self) -> Option<PxRigidDynamicRef>;
    fn to_rigid_actor(&self) -> Option<PxRigidActorRef>;
    fn to_rigid_body(&self) -> Option<PxRigidBodyRef>;
    fn to_rigid_static(&self) -> Option<PxRigidStaticRef>;
    fn to_joint(&self) -> Option<PxJointRef>;
    fn to_fixed_joint(&self) -> Option<PxFixedJointRef>;
    fn to_revolute_joint(&self) -> Option<PxRevoluteJointRef>;
}
impl as_any::Downcast for dyn PxBase {}
impl as_any::Downcast for dyn PxBase + Send {}
impl as_any::Downcast for dyn PxBase + Sync {}
impl as_any::Downcast for dyn PxBase + Send + Sync {}
impl<T: AsPxBase + 'static> PxBase for T {
    fn get_concrete_type(&self) -> u16 {
        unsafe { physx_sys::PxBase_getConcreteType(self.as_base().0) }
    }
    fn as_px_any(&self) -> PxAny {
        PxAny::from_obj(self.as_base().0)
    }
    fn to_rigid_dynamic(&self) -> Option<PxRigidDynamicRef> {
        match self.as_px_any() {
            PxAny::PxRigidDynamic(o) => Some(o),
            _ => None,
        }
    }
    fn to_rigid_actor(&self) -> Option<PxRigidActorRef> {
        match self.as_px_any() {
            PxAny::PxRigidDynamic(o) => Some(o.as_rigid_actor()),
            PxAny::PxRigidStatic(o) => Some(o.as_rigid_actor()),
            _ => None,
        }
    }
    fn to_rigid_body(&self) -> Option<PxRigidBodyRef> {
        match self.as_px_any() {
            PxAny::PxRigidDynamic(o) => Some(o.as_rigid_body()),
            PxAny::PxArticulationLink(o) => Some(o.as_rigid_body()),
            _ => None,
        }
    }
    fn to_rigid_static(&self) -> Option<PxRigidStaticRef> {
        match self.as_px_any() {
            PxAny::PxRigidStatic(o) => Some(o),
            _ => None,
        }
    }
    fn to_joint(&self) -> Option<PxJointRef> {
        match self.as_px_any() {
            PxAny::PxFixedJoint(o) => Some(o.as_joint()),
            PxAny::PxRevoluteJoint(o) => Some(o.as_joint()),
            _ => None,
        }
    }
    fn to_fixed_joint(&self) -> Option<PxFixedJointRef> {
        match self.as_px_any() {
            PxAny::PxFixedJoint(o) => Some(o),
            _ => None,
        }
    }
    fn to_revolute_joint(&self) -> Option<PxRevoluteJointRef> {
        match self.as_px_any() {
            PxAny::PxRevoluteJoint(o) => Some(o),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PxBaseRef(pub(crate) *mut physx_sys::PxBase);
impl AsPxBase for PxBaseRef {
    fn as_base(&self) -> PxBaseRef {
        PxBaseRef(self.0)
    }
}
unsafe impl Sync for PxBaseRef {}
unsafe impl Send for PxBaseRef {}

#[derive(Debug)]
pub enum PxAny {
    PxAggregate(PxAggregateRef),
    PxHeightField(PxHeightField),
    PxConvexMesh(PxConvexMesh),
    PxMaterial(PxMaterial),
    PxRigidDynamic(PxRigidDynamicRef),
    PxRigidStatic(PxRigidStaticRef),
    PxShape(PxShape),
    PxFixedJoint(PxFixedJointRef),
    PxRevoluteJoint(PxRevoluteJointRef),
    PxConstraint(PxConstraintRef),
    PxArticulationLink(PxArticulationLinkRef),
}
impl PxAny {
    pub(crate) fn from_obj(obj: *mut physx_sys::PxBase) -> Self {
        unsafe {
            match physx_sys::PxBase_getConcreteType(obj) as u32 {
                physx_sys::PxConcreteType::eAGGREGATE => PxAny::PxAggregate(PxAggregateRef(obj as _)),
                physx_sys::PxConcreteType::eARTICULATION => todo!(),
                physx_sys::PxConcreteType::eARTICULATION_JOINT => todo!(),
                physx_sys::PxConcreteType::eARTICULATION_JOINT_REDUCED_COORDINATE => todo!(),
                physx_sys::PxConcreteType::eARTICULATION_LINK => PxAny::PxArticulationLink(PxArticulationLinkRef(obj as _)),
                physx_sys::PxConcreteType::eARTICULATION_REDUCED_COORDINATE => todo!(),
                physx_sys::PxConcreteType::eBVH_STRUCTURE => todo!(),
                physx_sys::PxConcreteType::eCONSTRAINT => PxAny::PxConstraint(PxConstraintRef(obj as _)),
                physx_sys::PxConcreteType::eCONVEX_MESH => PxAny::PxConvexMesh(PxConvexMesh::from_ptr(obj as _)),
                physx_sys::PxConcreteType::eFIRST_USER_EXTENSION => todo!(),
                physx_sys::PxConcreteType::eFIRST_VEHICLE_EXTENSION => todo!(),
                physx_sys::PxConcreteType::eHEIGHTFIELD => PxAny::PxHeightField(PxHeightField::from_ptr(obj as _)),
                physx_sys::PxConcreteType::eMATERIAL => PxAny::PxMaterial(PxMaterial::from_ptr(obj as _)),
                physx_sys::PxConcreteType::ePHYSX_CORE_COUNT => todo!(),
                physx_sys::PxConcreteType::ePRUNING_STRUCTURE => todo!(),
                physx_sys::PxConcreteType::eRIGID_DYNAMIC => PxAny::PxRigidDynamic(PxRigidDynamicRef(obj as _)),
                physx_sys::PxConcreteType::eRIGID_STATIC => PxAny::PxRigidStatic(PxRigidStaticRef(obj as _)),
                physx_sys::PxConcreteType::eSHAPE => PxAny::PxShape(PxShape::from_ptr(obj as _)),
                physx_sys::PxConcreteType::eTRIANGLE_MESH_BVH33 => todo!(),
                physx_sys::PxConcreteType::eTRIANGLE_MESH_BVH34 => todo!(),

                physx_sys::PxJointConcreteType::eCONTACT => todo!(),
                physx_sys::PxJointConcreteType::eD6 => todo!(),
                physx_sys::PxJointConcreteType::eDISTANCE => todo!(),
                physx_sys::PxJointConcreteType::eFIXED => PxAny::PxFixedJoint(PxFixedJointRef(obj as _)),
                physx_sys::PxJointConcreteType::eLast => todo!(),
                physx_sys::PxJointConcreteType::ePRISMATIC => todo!(),
                physx_sys::PxJointConcreteType::eREVOLUTE => PxAny::PxRevoluteJoint(PxRevoluteJointRef(obj as _)),
                physx_sys::PxJointConcreteType::eSPHERICAL => todo!(),

                _ => panic!("Unknown type"),
            }
        }
    }
}
