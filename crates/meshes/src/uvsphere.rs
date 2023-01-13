use std::f32::consts::PI;

use elements_std::mesh::Mesh;
use glam::{vec2, vec3, Vec2, Vec3};

/// A sphere made of sectors and stacks
#[derive(Debug, Clone, Copy)]
pub struct UVSphereMesh {
    /// The radius of the sphere.
    pub radius: f32,
    /// Longitudinal sectors
    pub sectors: usize,
    /// Latitudinal stacks
    pub stacks: usize,
}

impl Default for UVSphereMesh {
    fn default() -> Self {
        Self { radius: 1.0, sectors: 36, stacks: 18 }
    }
}
impl From<UVSphereMesh> for Mesh {
    fn from(sphere: UVSphereMesh) -> Self {
        // From: https://github.com/bevyengine/bevy/blob/cf221f9659127427c99d621b76c8085c4860e2ef/crates/bevy_render/src/mesh/shape/uvsphere.rs

        let sectors = sphere.sectors as f32;
        let stacks = sphere.stacks as f32;
        let length_inv = 1. / sphere.radius;
        let sector_step = 2. * PI / sectors;
        let stack_step = PI / stacks;

        let mut vertices: Vec<Vec3> = Vec::with_capacity(sphere.stacks * sphere.sectors);
        let mut normals: Vec<Vec3> = Vec::with_capacity(sphere.stacks * sphere.sectors);
        let mut uvs: Vec<Vec2> = Vec::with_capacity(sphere.stacks * sphere.sectors);
        let mut indices: Vec<u32> = Vec::with_capacity(sphere.stacks * sphere.sectors * 2 * 3);

        for i in 0..sphere.stacks + 1 {
            let stack_angle = PI / 2. - (i as f32) * stack_step;
            let xy = sphere.radius * stack_angle.cos();
            let z = sphere.radius * stack_angle.sin();

            for j in 0..sphere.sectors + 1 {
                let sector_angle = (j as f32) * sector_step;
                let x = xy * sector_angle.cos();
                let y = xy * sector_angle.sin();

                vertices.push(vec3(x, y, z));
                normals.push(vec3(x * length_inv, y * length_inv, z * length_inv));
                uvs.push(vec2((j as f32) / sectors, (i as f32) / stacks));
            }
        }

        // indices
        //  k1--k1+1
        //  |  / |
        //  | /  |
        //  k2--k2+1
        for i in 0..sphere.stacks {
            let mut k1 = i * (sphere.sectors + 1);
            let mut k2 = k1 + sphere.sectors + 1;
            for _j in 0..sphere.sectors {
                if i != 0 {
                    indices.push(k1 as u32);
                    indices.push(k2 as u32);
                    indices.push((k1 + 1) as u32);
                }
                if i != sphere.stacks - 1 {
                    indices.push((k1 + 1) as u32);
                    indices.push(k2 as u32);
                    indices.push((k2 + 1) as u32);
                }
                k1 += 1;
                k2 += 1;
            }
        }

        let mut mesh = Mesh {
            name: format!("{sphere:?}"),
            positions: Some(vertices),
            texcoords: vec![uvs],
            normals: Some(normals),
            indices: Some(indices),
            ..Default::default()
        };
        mesh.create_tangents();
        mesh
    }
}
