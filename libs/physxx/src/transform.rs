use std::fmt;

#[derive(Clone, Copy)]
pub struct PxTransform(pub(crate) physx_sys::PxTransform);
impl PxTransform {
    pub fn from_translation(translation: glam::Vec3) -> Self {
        Self(unsafe { physx_sys::PxTransform_new_1(&to_physx_vec3(translation)) })
    }
    pub fn new(translation: glam::Vec3, rotation: glam::Quat) -> Self {
        Self(unsafe { physx_sys::PxTransform_new_5(&to_physx_vec3(translation), &to_physx_quat(rotation)) })
    }
    pub fn from_rotation(rotation: glam::Quat) -> Self {
        Self(unsafe { physx_sys::PxTransform_new_3(&to_physx_quat(rotation)) })
    }
    pub fn identity() -> Self {
        Self(unsafe { physx_sys::PxTransform_new_2(physx_sys::PxIdentity) })
    }
    pub fn translation(&self) -> glam::Vec3 {
        to_glam_vec3(&self.0.p)
    }
    pub fn rotation(&self) -> glam::Quat {
        to_glam_quat(self.0.q)
    }
    pub fn to_mat4(&self) -> glam::Mat4 {
        glam::Mat4::from_rotation_translation(self.rotation(), self.translation())
    }
    pub fn set_translation(&mut self, translation: glam::Vec3) {
        self.0.p = to_physx_vec3(translation);
    }
    pub fn set_rotation(&mut self, rotation: glam::Quat) {
        self.0.q = to_physx_quat(rotation);
    }
}
impl fmt::Debug for PxTransform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Transform").field("translation", &self.translation()).field("rotation", &self.rotation()).finish()
    }
}

pub(crate) fn to_glam_vec3(v: &physx_sys::PxVec3) -> glam::Vec3 {
    glam::Vec3::new(v.x, v.y, v.z)
}
pub(crate) fn to_glam_vec3_f64(v: &physx_sys::PxExtendedVec3) -> glam::f64::DVec3 {
    glam::f64::DVec3::new(v.x, v.y, v.z)
}
pub(crate) fn to_glam_quat(v: physx_sys::PxQuat) -> glam::Quat {
    glam::Quat::from_xyzw(v.x, v.y, v.z, v.w)
}
pub(crate) fn to_physx_vec3(v: glam::Vec3) -> physx_sys::PxVec3 {
    physx_sys::PxVec3 { x: v.x, y: v.y, z: v.z }
}
pub(crate) fn to_physx_vec3_f64(v: glam::f64::DVec3) -> physx_sys::PxExtendedVec3 {
    physx_sys::PxExtendedVec3 { x: v.x, y: v.y, z: v.z }
}
pub(crate) fn to_physx_quat(v: glam::Quat) -> physx_sys::PxQuat {
    physx_sys::PxQuat { x: v.x, y: v.y, z: v.z, w: v.w }
}
