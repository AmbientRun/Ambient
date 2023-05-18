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

pub fn create(desc: &Descriptor) -> ProceduralMeshHandle {
    let vertices = unsafe {
        std::slice::from_raw_parts(
            desc.vertices.as_ptr().cast::<wit::client_mesh::Vertex>(),
            desc.vertices.len(),
        )
    };
    wit::client_mesh::create(wit::client_mesh::Descriptor {
        vertices,
        indices: desc.indices,
    })
    .from_bindgen()
}

pub fn destroy(handle: ProceduralMeshHandle) {
    wit::client_mesh::destroy(handle.into_bindgen());
}
