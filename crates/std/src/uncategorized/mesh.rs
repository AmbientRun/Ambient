use crate::shapes::AABB;
use anyhow::ensure;
use glam::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default)]
pub struct MeshBuilder {
    pub positions: Vec<Vec3>,
    pub colors: Vec<Vec4>,
    pub normals: Vec<Vec3>,
    pub tangents: Vec<Vec3>,
    pub texcoords: Vec<Vec<Vec2>>,
    pub joint_indices: Vec<UVec4>,
    pub joint_weights: Vec<Vec4>,
    pub indices: Vec<u32>,
}

impl MeshBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self) -> anyhow::Result<Mesh> {
        ensure!(!self.positions.is_empty());
        ensure!(!self.indices.is_empty());
        ensure!(self.colors.is_empty() || self.positions.len() == self.colors.len());
        ensure!(self.normals.is_empty() || self.positions.len() == self.normals.len());
        ensure!(self.tangents.is_empty() || self.positions.len() == self.tangents.len());
        ensure!(
            self.texcoords.is_empty()
                || self
                    .texcoords
                    .iter()
                    .all(|tc| tc.len() == self.positions.len())
        );

        let mut aabb: crate::shapes::Cuboid = AABB::new_invalid();
        for &position in &self.positions {
            aabb.take_point(position);
        }

        Ok(Mesh {
            positions: self.positions,
            colors: self.colors,
            normals: self.normals,
            tangents: self.tangents,
            texcoords: self.texcoords,
            joint_indices: self.joint_indices,
            joint_weights: self.joint_weights,
            indices: self.indices,
            aabb,
        })
    }
}

pub fn generate_tangents(
    positions: &[Vec3],
    texcoords: &[Vec2],
    normals: &[Vec3],
    indices: &[u32],
) -> Vec<Vec3> {
    struct Geometry<'a> {
        positions: &'a [Vec3],
        texcoords: &'a [Vec2],
        normals: &'a [Vec3],
        indices: &'a [u32],
        tangents: &'a mut [Vec3],
    }

    impl Geometry<'_> {
        fn index(&self, face: usize, vert: usize) -> usize {
            self.indices[3 * face + vert] as _
        }
    }

    impl mikktspace::Geometry for Geometry<'_> {
        fn num_faces(&self) -> usize {
            self.indices.len() / 3
        }

        fn num_vertices_of_face(&self, _face: usize) -> usize {
            3
        }

        fn position(&self, face: usize, vert: usize) -> [f32; 3] {
            self.positions[self.index(face, vert)].into()
        }

        fn normal(&self, face: usize, vert: usize) -> [f32; 3] {
            self.normals[self.index(face, vert)].into()
        }

        fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
            self.texcoords[self.index(face, vert)].into()
        }

        fn set_tangent(
            &mut self,
            tangent: [f32; 3],
            _bi_tangent: [f32; 3],
            _f_mag_s: f32,
            _f_mag_t: f32,
            _bi_tangent_preserves_orientation: bool,
            face: usize,
            vert: usize,
        ) {
            self.tangents[self.index(face, vert)] = tangent.into();
        }
    }

    let mut tangents = vec![Vec3::ZERO; positions.len()];
    let mut geometry = Geometry {
        positions,
        texcoords,
        normals,
        indices,
        tangents: &mut tangents,
    };
    let result = mikktspace::generate_tangents(&mut geometry);
    if !result {
        log::warn!("mikktspace::generate_tangents failed");
    }
    tangents
}

pub fn flip_winding(indices: &mut [u32]) {
    for chunk in indices.chunks_exact_mut(3) {
        chunk.swap(1, 2);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mesh {
    positions: Vec<Vec3>,
    colors: Vec<Vec4>,
    normals: Vec<Vec3>,
    tangents: Vec<Vec3>,
    texcoords: Vec<Vec<Vec2>>,
    joint_indices: Vec<UVec4>,
    joint_weights: Vec<Vec4>,
    indices: Vec<u32>,
    aabb: AABB,
}

impl Mesh {
    pub fn positions(&self) -> &[Vec3] {
        &self.positions
    }

    pub fn colors(&self) -> &[Vec4] {
        &self.colors
    }

    pub fn normals(&self) -> &[Vec3] {
        &self.normals
    }

    pub fn tangents(&self) -> &[Vec3] {
        &self.tangents
    }

    pub fn texcoords(&self, set: usize) -> &[Vec2] {
        &self.texcoords[set]
    }

    pub fn joint_indices(&self) -> &[UVec4] {
        &self.joint_indices
    }

    pub fn joint_weights(&self) -> &[Vec4] {
        &self.joint_weights
    }

    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    pub fn index_count(&self) -> u32 {
        self.indices.len() as _
    }

    pub fn aabb(&self) -> AABB {
        self.aabb
    }

    pub fn size_in_bytes(&self) -> usize {
        let mut byte_size = 0;
        byte_size += std::mem::size_of_val(self.positions.as_slice());
        byte_size += std::mem::size_of_val(self.colors.as_slice());
        byte_size += std::mem::size_of_val(self.normals.as_slice());
        byte_size += std::mem::size_of_val(self.tangents.as_slice());
        byte_size += std::mem::size_of_val(self.joint_indices.as_slice());
        byte_size += std::mem::size_of_val(self.joint_weights.as_slice());
        byte_size += std::mem::size_of_val(self.indices.as_slice());
        byte_size += self
            .texcoords
            .iter()
            .map(|x| std::mem::size_of_val(x.as_slice()))
            .sum::<usize>();

        byte_size
    }

    pub fn into_geometry<F, T>(self, f: F) -> T
    where
        F: FnOnce(Vec<Vec3>, Vec<u32>) -> T,
    {
        f(self.positions, self.indices)
    }

    pub fn transformed(mut self, transform: Mat4) -> Self {
        self.aabb = AABB::new_invalid();
        for p in &mut self.positions {
            *p = transform.project_point3(*p);
            self.aabb.take_point(*p);
        }
        for n in &mut self.normals {
            *n = transform.transform_vector3(*n).normalize();
        }
        self
    }

    pub fn winding_flipped(mut self) -> Self {
        flip_winding(&mut self.indices);
        self
    }
}
