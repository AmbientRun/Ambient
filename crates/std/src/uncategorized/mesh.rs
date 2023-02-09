use glam::*;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::shapes::AABB;

#[derive(Clone, Serialize, Deserialize)]
pub struct Mesh {
    pub name: String,
    pub positions: Option<Vec<Vec3>>,
    pub colors: Option<Vec<Vec4>>,
    pub normals: Option<Vec<Vec3>>,
    pub tangents: Option<Vec<Vec3>>,
    pub texcoords: Vec<Vec<Vec2>>,
    pub joint_indices: Option<Vec<UVec4>>,
    pub joint_weights: Option<Vec<Vec4>>,
    pub indices: Option<Vec<u32>>,
}

impl std::fmt::Debug for Mesh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mesh")
            .field("name", &self.name)
            .field("positions", &self.positions.as_ref().map(|v| v.len()).unwrap_or_default())
            .field("colors", &self.colors.as_ref().map(|v| v.len()).unwrap_or_default())
            .field("normals", &self.normals.as_ref().map(|v| v.len()).unwrap_or_default())
            .field("tangents", &self.tangents.as_ref().map(|v| v.len()).unwrap_or_default())
            .field("texcoords", &self.texcoords.iter().map(|v| v.len()).collect_vec())
            .field("joint_indices", &self.joint_indices.as_ref().map(|v| v.len()).unwrap_or_default())
            .field("joint_weights", &self.joint_weights.as_ref().map(|v| v.len()).unwrap_or_default())
            .field("indices", &self.indices.as_ref().map(|v| v.len()).unwrap_or_default())
            .finish()
    }
}

impl Default for Mesh {
    fn default() -> Self {
        Self {
            name: "Unnamed".to_string(),
            positions: None,
            colors: None,
            normals: None,
            tangents: None,
            texcoords: Vec::new(),
            joint_indices: None,
            joint_weights: None,
            indices: None,
        }
    }
}
impl Mesh {
    pub fn aabb(&self) -> Option<AABB> {
        if let Some(positions) = &self.positions {
            if positions.is_empty() {
                return None;
            }
            let mut aabb = AABB { min: positions[0], max: positions[0] };
            for &pos in positions.iter().skip(1) {
                aabb.min = aabb.min.min(pos);
                aabb.max = aabb.max.max(pos);
            }
            Some(aabb)
        } else {
            None
        }
    }
    pub fn transform(&mut self, transform: Mat4) {
        if let Some(positions) = &mut self.positions {
            for p in positions {
                *p = transform.project_point3(vec3(p[0], p[1], p[2]));
            }
        }
        if let Some(normals) = &mut self.normals {
            for p in normals {
                *p = transform.transform_vector3(vec3(p[0], p[1], p[2])).normalize();
            }
        }
    }
    /// Flips indicies so that what was front facing will become back facing and vice versa
    pub fn invert_indicies(&mut self) {
        if let Some(indicies) = &mut self.indices {
            for chunk in indicies.chunks_exact_mut(3) {
                chunk.swap(1, 2);
            }
        }
    }
    pub fn apply_skin(&mut self, joint_matrices: &[Mat4]) {
        if let (Some(positions), Some(weights), Some(indices)) = (&mut self.positions, &self.joint_weights, &self.joint_indices) {
            for ((position, weight), index) in positions.iter_mut().zip(weights.iter()).zip(indices.iter()) {
                let mat = joint_matrices[index.x as usize] * weight.x
                    + joint_matrices[index.y as usize] * weight.y
                    + joint_matrices[index.z as usize] * weight.z
                    + joint_matrices[index.w as usize] * weight.w;
                *position = mat.transform_point3(*position);
            }
        }
    }

    #[profiling::function]
    pub fn append(&mut self, mut mesh: Mesh) {
        let indices_offset = self.positions.as_ref().unwrap().len() as u32;
        if let Some(x) = &mut self.positions {
            x.extend(mesh.positions.unwrap());
        }
        if let Some(x) = &mut self.colors {
            x.extend(mesh.colors.unwrap());
        }
        if let Some(x) = &mut self.normals {
            x.extend(mesh.normals.unwrap());
        }
        for (i, x) in self.texcoords.iter_mut().enumerate() {
            x.append(&mut mesh.texcoords[i]);
        }
        if let Some(x) = &mut self.joint_indices {
            x.extend(mesh.joint_indices.unwrap());
        }
        if let Some(x) = &mut self.joint_weights {
            x.extend(mesh.joint_weights.unwrap());
        }
        if let Some(x) = &mut self.indices {
            x.extend(mesh.indices.unwrap().into_iter().map(|i| i + indices_offset));
        }
    }

    #[profiling::function]
    pub fn remove_unused_vertices(&mut self) {
        let mut used = Vec::new();
        let mut new_indices = Vec::new();
        let n_vertices = self.positions.as_ref().unwrap().len();
        used.resize(n_vertices, false);
        new_indices.resize(n_vertices, 0);
        for i in self.indices.as_ref().unwrap().iter() {
            used[*i as usize] = true;
        }
        let mut p = 0;
        for i in 0..n_vertices {
            new_indices[i] = p;
            if used[i] {
                p += 1;
            }
        }
        for index in self.indices.as_mut().unwrap().iter_mut() {
            *index = new_indices[*index as usize];
        }
        self.positions = self
            .positions
            .as_mut()
            .map(|positions| positions.drain(..).enumerate().filter_map(|(i, v)| if used[i] { Some(v) } else { None }).collect());
        self.colors = self
            .colors
            .as_mut()
            .map(|colors| colors.drain(..).enumerate().filter_map(|(i, v)| if used[i] { Some(v) } else { None }).collect());
        self.normals = self
            .normals
            .as_mut()
            .map(|normals| normals.drain(..).enumerate().filter_map(|(i, v)| if used[i] { Some(v) } else { None }).collect());
        self.joint_indices = self
            .joint_indices
            .as_mut()
            .map(|joints| joints.drain(..).enumerate().filter_map(|(i, v)| if used[i] { Some(v) } else { None }).collect());
        self.joint_weights = self
            .joint_weights
            .as_mut()
            .map(|weights| weights.drain(..).enumerate().filter_map(|(i, v)| if used[i] { Some(v) } else { None }).collect());
        self.texcoords = self
            .texcoords
            .drain(..)
            .map(|texcoords| texcoords.into_iter().enumerate().filter_map(|(i, v)| if used[i] { Some(v) } else { None }).collect())
            .collect();
    }
    pub fn try_ensure_tangents(&mut self) {
        if self.tangents.is_some() || self.positions.is_none() || self.texcoords.is_empty() {
            log::info!("Tangents loaded from mesh");
            return;
        }
        self.create_tangents();
    }

    #[profiling::function]
    pub fn create_tangents(&mut self) {
        let mut tangents = vec![Vec3::ZERO; self.positions.as_ref().unwrap().len()];
        let mut tangent_counts = vec![0.; self.positions.as_ref().unwrap().len()];
        let positions = self.positions.as_ref().expect("Can not create tangents without position data");
        let texcoords = &self.texcoords[0];

        for triangle in self.indices.as_ref().unwrap().chunks(3) {
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

            tangent_counts[triangle[0] as usize] += 1.;
            tangent_counts[triangle[1] as usize] += 1.;
            tangent_counts[triangle[2] as usize] += 1.;
        }
        for i in 0..tangents.len() {
            tangents[i] /= tangent_counts[i];
        }
        self.tangents = Some(tangents);
    }
    pub fn size_in_bytes(&self) -> usize {
        self.positions.as_ref().map(|x| std::mem::size_of_val(&**x)).unwrap_or(0)
            + self.colors.as_ref().map(|x| std::mem::size_of_val(&**x)).unwrap_or(0)
            + self.normals.as_ref().map(|x| std::mem::size_of_val(&**x)).unwrap_or(0)
            + self.tangents.as_ref().map(|x| std::mem::size_of_val(&**x)).unwrap_or(0)
            + self.joint_indices.as_ref().map(|x| std::mem::size_of_val(&**x)).unwrap_or(0)
            + self.joint_weights.as_ref().map(|x| std::mem::size_of_val(&**x)).unwrap_or(0)
            + self.indices.as_ref().map(|x| std::mem::size_of_val(&**x)).unwrap_or(0)
            + self.texcoords.iter().map(|x| std::mem::size_of_val(&**x)).sum::<usize>()
    }
}
