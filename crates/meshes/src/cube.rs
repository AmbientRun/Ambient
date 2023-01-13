use std::hash::Hash;

use elements_std::mesh::Mesh;
use glam::*;

use crate::cuboid::CuboidMesh;

#[derive(Debug, Clone)]
pub struct CubeMesh {
    pub position: Vec3,
    pub size: Vec3,
    pub color: Vec4,
}
impl CubeMesh {
    pub fn from_size(size: Vec3) -> Self {
        Self { size, position: -size / 2., color: vec4(1., 1., 1., 1.) }
    }
}
impl PartialEq for CubeMesh {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position && self.size == other.size && self.color == other.color
    }
}
impl Eq for CubeMesh {}
impl Hash for CubeMesh {
    fn hash<H>(&self, state: &mut H)
    where
        H: ::std::hash::Hasher,
    {
        format!("{self:?}").hash(state);
    }
}
impl Default for CubeMesh {
    fn default() -> Self {
        Self { position: vec3(-1., -1., -1.), size: vec3(2.0, 2.0, 2.0), color: vec4(1.0, 1.0, 1.0, 1.0) }
    }
}

impl From<CubeMesh> for Mesh {
    fn from(box3: CubeMesh) -> Mesh {
        From::from(&box3)
    }
}
impl From<&CubeMesh> for Mesh {
    fn from(cube: &CubeMesh) -> Mesh {
        let min = cube.position;
        let max = cube.position + cube.size;
        let mut mesh = Mesh::from(&CuboidMesh {
            // Order: Bottom [ Left (Back, Front), Right (Back, Front) ] - Top [ Left (Back, Front), Right (Back, Front) ]
            positions: [
                vec3(min.x, min.y, min.z),
                vec3(max.x, min.y, min.z),
                vec3(min.x, max.y, min.z),
                vec3(max.x, max.y, min.z),
                vec3(min.x, min.y, max.z),
                vec3(max.x, min.y, max.z),
                vec3(min.x, max.y, max.z),
                vec3(max.x, max.y, max.z),
            ],
            color: Some(cube.color),
            texcoords: true,
            normals: true,
            tangents: true,
        });
        mesh.name = "cube".to_string();
        mesh
    }
}
