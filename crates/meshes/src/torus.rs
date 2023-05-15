use ambient_std::mesh::{generate_tangents, Mesh, MeshBuilder};
use glam::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TorusMesh {
    pub inner_radius: f32,
    pub outer_radius: f32,
    pub slices: u32,
    pub loops: u32,
}

impl Default for TorusMesh {
    fn default() -> Self {
        TorusMesh {
            inner_radius: 0.25,
            outer_radius: 1.0,
            slices: 16,
            loops: 16,
        }
    }
}

impl From<TorusMesh> for Mesh {
    fn from(torus: TorusMesh) -> Self {
        let TorusMesh {
            inner_radius,
            outer_radius,
            slices,
            loops,
        } = torus;

        let vertex_count = ((slices + 1) * (loops + 1)) as usize;
        let mut vertices = Vec::with_capacity(vertex_count);
        let mut normals = Vec::with_capacity(vertex_count);
        let mut texcoords = Vec::with_capacity(vertex_count);

        let ring_factor = std::f32::consts::PI * 2.0 / slices as f32;
        let loop_factor = std::f32::consts::PI * 2.0 / loops as f32;

        for i in 0..=loops {
            let u = i as f32 * loop_factor;
            let cos_u = u.cos();
            let sin_u = u.sin();

            for j in 0..=slices {
                let v = j as f32 * ring_factor;
                let cos_v = v.cos();
                let sin_v = v.sin();

                let r = outer_radius + inner_radius * cos_v;
                let x = r * cos_u;
                let y = r * sin_u;
                let z = outer_radius * sin_v / 2.0;

                vertices.push(Vec3::new(x, y, z));

                let nv = Vec3::new(cos_v * cos_u, cos_v * sin_u, sin_v);
                normals.push(nv.normalize());

                texcoords.push(Vec2::new(v / ring_factor, 1.0 - u / loop_factor));
            }
        }

        let index_count = (slices * loops * 6) as usize;
        let mut indices = Vec::with_capacity(index_count);

        for i in 0..loops {
            for j in 0..slices {
                let a = i * (slices + 1) + j;
                let b = a + slices + 1;

                indices.push(a);
                indices.push(b);
                indices.push(a + 1);

                indices.push(b);
                indices.push(b + 1);
                indices.push(a + 1);
            }
        }

        let tangents = generate_tangents(&vertices, &texcoords, &indices);
        MeshBuilder {
            positions: vertices,
            normals,
            tangents,
            texcoords: vec![texcoords],
            indices,
            ..MeshBuilder::default()
        }
        .build()
        .expect("Invalid torus mesh")
    }
}
