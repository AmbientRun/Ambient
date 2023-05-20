use ambient_std::mesh::{generate_tangents, Mesh, MeshBuilder};
use glam::*;
use rand::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TreeMesh {
    pub seed: i32,
    pub trunk_radius: f32,
    pub trunk_height: f32,
    pub trunk_segments: u32,
    pub branch_length: f32,
    pub branch_angle: f32,
    pub branch_segments: u32,
    pub foliage_radius: f32,
    pub foliage_density: u32,
    pub foliage_segments: u32,
}

impl Default for TreeMesh {
    fn default() -> Self {
        Self {
            seed: 123 as i32,
            trunk_radius: 0.2,
            trunk_height: 1.0,
            trunk_segments: 5,
            branch_length: 0.5,
            branch_angle: std::f32::consts::PI / 4.0,
            branch_segments: 3,
            foliage_radius: 0.5,
            foliage_density: 5,
            foliage_segments: 5,
        }
    }
}

impl From<TreeMesh> for Mesh {
    fn from(tree: TreeMesh) -> Self {
        let tree_mesh = tree;

        // Create the trunk
        let (mut vertices1, top_vertices1, mut normals1, mut uvs1) = build_trunk(&tree_mesh);

        let sectors = 12;
        let trunk_segments = tree_mesh.trunk_segments;
        let _branch_segments = tree_mesh.branch_segments;
        let mut indices = Vec::new();

        // Connect trunk segments
        for i in 0..(trunk_segments) {
            for j in 0..sectors {
                let k1 = (i * (sectors as u32 + 1) + j as u32) as u32;
                let k2 = ((i + 1) * (sectors as u32 + 1) + j as u32) as u32;

                indices.push(k1);
                indices.push(k1 + 1);
                indices.push(k2);

                indices.push(k1 + 1);
                indices.push(k2 + 1);
                indices.push(k2);
            }
        }

        // Generate foliage
        let foliage_count = tree.foliage_density + tree.foliage_segments;
        let foliage_radius_variance = 0.05;
        let foliage_position_variance = vec3(56.0, 56.0, 36.0);

        for i in 0..foliage_count {
            let foliage_radius = tree.foliage_radius
                * (1.0 - gen_rn(tree.seed + i as i32, 0.0, 1.0) * foliage_radius_variance);
            let foliage_position = top_vertices1
                [gen_rn(tree.seed, 0.0, top_vertices1.len() as f32) as usize]
                + vec3(
                    gen_rn(tree.seed + i as i32, 0.0, 1.0) * foliage_position_variance.x,
                    gen_rn(tree.seed + i as i32 + 1, 0.0, 1.0) * foliage_position_variance.y,
                    gen_rn(tree.seed + i as i32 + 2, 0.0, 1.0) * foliage_position_variance.z + 2.0,
                );

            let segments = tree.foliage_segments;
            let density = tree.foliage_density;
            let sector_step = 2. * std::f32::consts::PI / density as f32;

            let mut sphere_vertices = Vec::new();
            let mut sphere_normals = Vec::new();
            let mut sphere_uvs = Vec::new();

            for i in 0..=segments {
                let theta = (i as f32 / segments as f32) * std::f32::consts::PI;
                let height = foliage_position.z + (foliage_radius * theta.cos());
                let segment_radius = foliage_radius * theta.sin();

                for j in 0..=density {
                    let phi = j as f32 * sector_step;
                    let x = foliage_position.x + segment_radius * phi.cos();
                    let y = foliage_position.y + segment_radius * phi.sin();
                    let z = height;

                    sphere_vertices.push(vec3(x, y, z));
                    sphere_normals.push(
                        vec3(
                            x - foliage_position.x,
                            y - foliage_position.y,
                            z - foliage_position.z,
                        )
                        .normalize(),
                    );
                    sphere_uvs.push(vec2(j as f32 / density as f32, i as f32 / segments as f32));
                }
            }

            let sphere_indices = generate_sphere_indices(segments as usize, density as usize);

            vertices1.extend(sphere_vertices.clone());
            normals1.extend(sphere_normals);
            uvs1.extend(sphere_uvs);

            let offset = vertices1.len() - sphere_vertices.len();
            indices.extend(sphere_indices.iter().map(|i| *i + offset as u32));
        }

        // Function to generate indices for a sphere based on segments and density
        fn generate_sphere_indices(segments: usize, density: usize) -> Vec<u32> {
            let mut indices = Vec::with_capacity(segments * density * 6);

            for i in 0..segments {
                for j in 0..density {
                    let index1 = i * (density + 1) + j;
                    let index2 = index1 + 1;
                    let index3 = (i + 1) * (density + 1) + j;
                    let index4 = index3 + 1;

                    indices.push(index1 as u32);
                    indices.push(index2 as u32);
                    indices.push(index3 as u32);

                    indices.push(index2 as u32);
                    indices.push(index4 as u32);
                    indices.push(index3 as u32);
                }
            }

            indices
        }

        let tangents = generate_tangents(&vertices1, &uvs1, &indices);
        MeshBuilder {
            positions: vertices1,
            normals: normals1,
            tangents: tangents,
            texcoords: vec![uvs1],
            indices: indices,
            ..MeshBuilder::default()
        }
        .build()
        .expect("Invalid tree mesh")
    }
}

fn build_trunk(tree: &TreeMesh) -> (Vec<Vec3>, Vec<Vec3>, Vec<Vec3>, Vec<Vec2>) {
    let sectors = 12;
    let sector_step = 2. * std::f32::consts::PI / sectors as f32;

    let mut vertices = Vec::new();
    let mut top_vertices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();

    let mut trunk_direction = vec3(0.0, 0.0, 1.0);
    let direction_variance = 0.08;

    let radius_variance = 0.02;

    for i in 0..=tree.trunk_segments {
        let variance = gen_rn(tree.seed + i as i32, 0.0, 1.0) * radius_variance;
        let z = tree.trunk_height * (i as f32 / tree.trunk_segments as f32);
        let s = tree.trunk_radius;
        let radius = s * (1.0 - i as f32 / tree.trunk_segments as f32) * (1.0 - variance);

        let top_segment_radius = tree.trunk_radius * 0.1;
        let radius = if i == tree.trunk_segments && radius < top_segment_radius {
            top_segment_radius
        } else {
            radius
        };

        let random_direction = vec3(
            gen_rn(tree.seed + i as i32 + 1, 0.0, 1.0) - 0.5,
            gen_rn(tree.seed + i as i32 + 2, 0.0, 1.0) - 0.5,
            gen_rn(tree.seed + i as i32 + 3, 0.0, 1.0) - 0.5,
        )
        .normalize()
            * direction_variance;
        trunk_direction = (trunk_direction + random_direction).normalize();

        let top_position = trunk_direction * z;

        let gravity_factor = (1.0 - (i as f32 / tree.trunk_segments as f32)).powf(2.0);
        let gravity_offset = trunk_direction * gravity_factor * 2.0 * i as f32;

        for j in 0..=sectors {
            let sector_angle = j as f32 * sector_step;
            let x = radius * sector_angle.cos();
            let y = radius * sector_angle.sin();

            vertices.push(top_position + vec3(x, y, 0.0) - gravity_offset);
            normals.push(vec3(x, y, 0.0).normalize());
            uvs.push(vec2(
                j as f32 / sectors as f32,
                i as f32 / tree.trunk_segments as f32,
            ));
        }

        if i == tree.trunk_segments {
            let top_vertex_start = vertices.len() - sectors - 1;
            let top_vertex_end = vertices.len();
            top_vertices.extend(vertices[top_vertex_start..top_vertex_end].iter().cloned());

            // Add faces to connect the last ring of vertices
            for j in 0..sectors {
                let v1 = top_vertex_start + j;
                let v2 = top_vertex_start + j + 1;
                let v3 = top_vertex_end - 1;
                let v4 = top_vertex_start;

                // First triangle
                vertices.push(vertices[v1] - gravity_offset);
                vertices.push(vertices[v2] - gravity_offset);
                vertices.push(vertices[v3] - gravity_offset);

                normals.push(normals[v1]);
                normals.push(normals[v2]);
                normals.push(normals[v3]);

                uvs.push(uvs[v1]);
                uvs.push(uvs[v2]);
                uvs.push(uvs[v3]);

                // Second triangle
                vertices.push(vertices[v1] - gravity_offset);
                vertices.push(vertices[v3] - gravity_offset);
                vertices.push(vertices[v4] - gravity_offset);

                normals.push(normals[v1]);
                normals.push(normals[v3]);
                normals.push(normals[v4]);

                uvs.push(uvs[v1]);
                uvs.push(uvs[v3]);
                uvs.push(uvs[v4]);
            }
        }
    }

    (vertices, top_vertices, normals, uvs)
}

pub fn gen_rn(seed: i32, min: f32, max: f32) -> f32 {
    let mut rng = ChaCha8Rng::seed_from_u64(seed as u64);
    rng.gen_range(min..max)
}
