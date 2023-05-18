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
