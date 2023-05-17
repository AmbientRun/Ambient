use crate::shapes::AABB;
use ambient_asset_cache::{AssetCache, SyncAssetKey, SyncAssetKeyExt};
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
            id: ulid::Ulid::new(),
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

pub fn generate_tangents(positions: &[Vec3], texcoords: &[Vec2], indices: &[u32]) -> Vec<Vec3> {
    let mut tangents = vec![Vec3::ZERO; positions.len()];
    let mut tangent_counts = vec![0.0; positions.len()];
    for triangle in indices.chunks(3) {
        let (a, b, c) = (triangle[0], triangle[1], triangle[2]);
        let pos0 = positions[a as usize];
        let pos1 = positions[b as usize];
        let pos2 = positions[c as usize];

        let uv0 = texcoords[a as usize];
        let uv1 = texcoords[b as usize];
        let uv2 = texcoords[c as usize];

        let d1 = pos1 - pos0;
        let d2 = pos2 - pos0;

        let dt1 = uv1 - uv0;
        let dt2 = uv2 - uv0;

        let tangent = (d1 * dt2.y - d2 * dt1.y).normalize_or_zero();

        tangents[triangle[0] as usize] += tangent;
        tangents[triangle[1] as usize] += tangent;
        tangents[triangle[2] as usize] += tangent;

        tangent_counts[triangle[0] as usize] += 1.0;
        tangent_counts[triangle[1] as usize] += 1.0;
        tangent_counts[triangle[2] as usize] += 1.0;
    }
    for i in 0..tangents.len() {
        tangents[i] /= tangent_counts[i];
    }
    tangents
}

pub fn flip_winding(indices: &mut [u32]) {
    for chunk in indices.chunks_exact_mut(3) {
        chunk.swap(1, 2);
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Mesh {
    id: ulid::Ulid,
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

impl std::fmt::Debug for Mesh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.id))
    }
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

#[derive(Debug)]
pub struct MeshKey(ulid::Ulid);

impl MeshKey {
    pub fn new() -> Self {
        Self(ulid::Ulid::new())
    }

    pub fn into_inner(self) -> ulid::Ulid {
        self.0
    }
}

impl std::str::FromStr for MeshKey {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(ulid::Ulid::from_str(s)?))
    }
}

impl std::fmt::Display for MeshKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl SyncAssetKey<Mesh> for MeshKey {
    fn load(&self, assets: AssetCache) -> Mesh {
        self.get(&assets)
    }
}
