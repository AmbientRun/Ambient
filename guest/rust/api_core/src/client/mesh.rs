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
impl IntoBindgen for Vertex {
    type Item = wit::client_mesh::Vertex;

    fn into_bindgen(self) -> Self::Item {
        Self::Item {
            position: self.position.into_bindgen(),
            normal: self.normal.into_bindgen(),
            tangent: self.tangent.into_bindgen(),
            texcoord0: self.texcoord0.into_bindgen(),
        }
    }
}

#[derive(Clone)]
pub struct Descriptor<'a> {
    pub vertices: &'a [Vertex],
    pub indices: &'a [u32],
}
impl<'a> IntoBindgen for &'a Descriptor<'a> {
    type Item = wit::client_mesh::Descriptor;

    fn into_bindgen(self) -> Self::Item {
        Self::Item {
            vertices: self.vertices.iter().map(|v| v.into_bindgen()).collect(),
            indices: self.indices.to_vec(),
        }
    }
}

pub fn create(desc: &Descriptor) -> ProceduralMeshHandle {
    wit::client_mesh::create(&desc.into_bindgen()).from_bindgen()
}

pub fn destroy(handle: ProceduralMeshHandle) {
    wit::client_mesh::destroy(handle.into_bindgen());
}
