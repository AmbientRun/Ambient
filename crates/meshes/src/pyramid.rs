use std::hash::Hash;

use ambient_std::mesh::{generate_tangents, Mesh, MeshBuilder};
use glam::*;

#[derive(Debug, Clone)]
pub struct PyramidMesh {
    pub position: Vec3,
    pub size: Vec3,
    pub color: Vec4,
}
impl PyramidMesh {
    pub fn from_size(size: Vec3) -> Self {
        Self {
            size,
            position: -size / 2.,
            color: vec4(1., 1., 1., 1.),
        }
    }
}
impl PartialEq for PyramidMesh {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position && self.size == other.size && self.color == other.color
    }
}
impl Eq for PyramidMesh {}
impl Hash for PyramidMesh {
    fn hash<H>(&self, state: &mut H)
    where
        H: ::std::hash::Hasher,
    {
        format!("{self:?}").hash(state);
    }
}
impl Default for PyramidMesh {
    fn default() -> Self {
        Self {
            position: vec3(-1., -1., -1.),
            size: vec3(2.0, 2.0, 2.0),
            color: vec4(1.0, 1.0, 1.0, 1.0),
        }
    }
}

impl From<PyramidMesh> for Mesh {
    fn from(box3: PyramidMesh) -> Mesh {
        From::from(&box3)
    }
}
impl From<&PyramidMesh> for Mesh {
    fn from(box3: &PyramidMesh) -> Mesh {
        let min = box3.position;
        let max = box3.position + box3.size;
        let top = vec3((max.x - min.x) / 2., (max.y - min.y) / 2., max.z);
        let positions = vec![
            //-Z
            vec3(min.x, min.y, min.z),
            vec3(max.x, min.y, min.z),
            vec3(min.x, max.y, min.z),
            vec3(max.x, max.y, min.z),
            //-X
            vec3(min.x, min.y, min.z),
            vec3(min.x, max.y, min.z),
            vec3(top.x, top.y, top.z),
            //+X
            vec3(max.x, max.y, min.z),
            vec3(max.x, min.y, min.z),
            vec3(top.x, top.y, top.z),
            //-Y
            vec3(max.x, min.y, min.z),
            vec3(min.x, min.y, min.z),
            vec3(top.x, top.y, top.z),
            //+Y
            vec3(min.x, max.y, min.z),
            vec3(max.x, max.y, min.z),
            vec3(top.x, top.y, top.z),
        ];

        let texcoords = vec![
            //-Z
            vec2(0.0, 0.0),
            vec2(0.0, 1.0),
            vec2(1.0, 0.0),
            vec2(1.0, 1.0),
            //-X
            vec2(0.0, 1.0),
            vec2(1.0, 1.0),
            vec2(1.0, 0.0),
            //+X
            vec2(0.0, 1.0),
            vec2(1.0, 1.0),
            vec2(1.0, 0.0),
            //-Y
            vec2(0.0, 1.0),
            vec2(1.0, 1.0),
            vec2(1.0, 0.0),
            //+Y
            vec2(0.0, 1.0),
            vec2(1.0, 1.0),
            vec2(1.0, 0.0),
        ];

        let normals = vec![
            //-Z
            vec3(0.0, 0.0, -1.0),
            vec3(0.0, 0.0, -1.0),
            vec3(0.0, 0.0, -1.0),
            vec3(0.0, 0.0, -1.0),
            //-X
            vec3(-1.0, 0.0, 0.0),
            vec3(-1.0, 0.0, 0.0),
            vec3(-1.0, 0.0, 0.0),
            //+X
            vec3(1.0, 0.0, 0.0),
            vec3(1.0, 0.0, 0.0),
            vec3(1.0, 0.0, 0.0),
            //-Y
            vec3(0.0, -1.0, 0.0),
            vec3(0.0, -1.0, 0.0),
            vec3(0.0, -1.0, 0.0),
            //+Y
            vec3(0.0, 1.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            vec3(0.0, 1.0, 0.0),
        ];

        let colors =
            std::iter::repeat(vec4(box3.color.x, box3.color.y, box3.color.z, box3.color.w))
                .take(24)
                .collect();

        let mut indices = vec![
            //-Z
            0, 2, 1, //
            //
            1, 2, 3,
        ];

        for i in 0..4 {
            indices.push(6 + i * 3);
            indices.push(6 + i * 3 + 2);
            indices.push(6 + i * 3 + 1);
        }
        let tangents = generate_tangents(&positions, &texcoords, &normals, &indices);
        MeshBuilder {
            positions,
            colors,
            normals,
            texcoords: vec![texcoords],
            tangents,
            indices,
            ..MeshBuilder::default()
        }
        .build()
        .expect("Invalid pyramid mesh")
    }
}
