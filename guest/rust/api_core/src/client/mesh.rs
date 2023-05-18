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

pub fn create(vertices: &[Vertex], indices: &[u32]) -> ProceduralMeshHandle {
    let vertices = unsafe {
        std::slice::from_raw_parts(
            vertices.as_ptr().cast::<wit::client_mesh::Vertex>(),
            vertices.len(),
        )
    };
    wit::client_mesh::create(vertices, indices).from_bindgen()
}
