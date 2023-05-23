use std::mem::size_of;

use static_assertions::const_assert_eq;

use crate::global::{ProceduralMeshHandle, Vec2, Vec3};
use crate::internal::conversion::*;
use crate::internal::wit;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub tangent: Vec3,
    pub texcoord0: Vec2,
}

#[derive(Clone)]
pub struct Descriptor<'a> {
    pub vertices: &'a [Vertex],
    pub indices: &'a [u32],
}

impl<'a> IntoBindgen for Descriptor<'a> {
    type Item = wit::client_mesh::Descriptor<'a>;

    fn into_bindgen(self) -> Self::Item {
        const_assert_eq!(size_of::<Vertex>(), size_of::<wit::client_mesh::Vertex>());
        Self::Item {
            vertices: unsafe {
                std::slice::from_raw_parts(
                    self.vertices.as_ptr().cast::<wit::client_mesh::Vertex>(),
                    self.vertices.len(),
                )
            },
            indices: self.indices,
        }
    }
}

pub fn create(desc: &Descriptor) -> ProceduralMeshHandle {
    wit::client_mesh::create(desc.clone().into_bindgen()).from_bindgen()
}

pub fn destroy(handle: ProceduralMeshHandle) {
    wit::client_mesh::destroy(handle.into_bindgen());
}
