use ambient_std::mesh::Mesh;
use glam::*;
use serde::{Deserialize, Serialize};
use rand::Rng;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TreeMesh {
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
        let sectors = 12;
        let sector_step = 2. * std::f32::consts::PI / sectors as f32;

        let mut vertices = Vec::new();
        let mut top_vertices = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut rng = rand::thread_rng();
        let mut indices = Vec::new();

        let branch_count = rng.gen_range(4..=7);

        let mut trunk_direction = vec3(0.0, 0.0, 1.0);
        let direction_variance = 0.08;

        let radius_variance = 0.02;
        let mut branch_radius = tree.trunk_radius * 0.5;

        // Generate trunk
        for i in 0..=tree.trunk_segments {

            let variance = rng.gen::<f32>() * radius_variance;
            let z = tree.trunk_height * (i as f32 / tree.trunk_segments as f32);
            let radius = tree.trunk_radius * (1.0 - i as f32 / tree.trunk_segments as f32) * (1.0 - variance);

            // Store current radius of the trunk for later use in the branches
            branch_radius = radius + 1.0;

            // Add some randomness to the direction of the trunk
            let random_direction = vec3(
                rng.gen::<f32>() - 0.5,
                rng.gen::<f32>() - 0.5,
                rng.gen::<f32>() - 0.5
            ).normalize() * direction_variance;
            trunk_direction = (trunk_direction + random_direction).normalize();

            let trunk_position = trunk_direction * z;

            for j in 0..=sectors {
                let sector_angle = j as f32 * sector_step;
                let x = radius * sector_angle.cos();
                let y = radius * sector_angle.sin();

                vertices.push(trunk_position + vec3(x, y, 0.0));
                normals.push(vec3(x, y, 0.0).normalize());
                uvs.push(vec2(j as f32 / sectors as f32, i as f32 / tree.trunk_segments as f32));
            }

            // Store the vertices at the top of the trunk for later
            if i == tree.trunk_segments {
                for _j in 0..=sectors {
                    top_vertices.push(vertices.last().unwrap().clone());
                }
            }
        }

        // Generate indices for trunk
        for i in 0..tree.trunk_segments {
            for j in 0..sectors {
                let k1 = (i * (sectors + 1) + j) as u32;
                let k2 = ((i + 1) * (sectors + 1) + j) as u32;

                indices.push(k1);
                indices.push(k1 + 1);
                indices.push(k2);

                indices.push(k1 + 1);
                indices.push(k2 + 1);
                indices.push(k2);
            }
        }
        // Generate branches
        let mut branch_bases = top_vertices.clone();
        for _ in 0..branch_count {
            let base_index = rng.gen_range(0..branch_bases.len());
            let branch_base = branch_bases[base_index];
            branch_bases.remove(base_index);

            let mut branch_direction = vec3(
                rng.gen::<f32>() - 0.5,
                rng.gen::<f32>() - 0.5,
                rng.gen::<f32>() - 0.5
            ).normalize();

            for i in 0..=tree.branch_segments {
                let variance = rng.gen::<f32>() * radius_variance;
                let z = tree.branch_length * (i as f32 / tree.branch_segments as f32);
                let radius = branch_radius * (1.0 - i as f32 / 2.0 / tree.branch_segments as f32) * (1.0 - variance);

                // Add some randomness to the direction of the branch
                let random_direction = vec3(
                    rng.gen::<f32>() - 0.5,
                    rng.gen::<f32>() - 0.5,
                    rng.gen::<f32>() - 0.5
                ).normalize() * direction_variance;
                branch_direction = (branch_direction + random_direction).normalize();

                let branch_position = branch_base + branch_direction * z;

                for j in 0..=sectors {
                    let sector_angle = j as f32 * sector_step;
                    let x = radius * sector_angle.cos();
                    let y = radius * sector_angle.sin();

                    vertices.push(branch_position + vec3(x, y, 0.0));
                    normals.push(vec3(x, y, 0.0).normalize());
                    uvs.push(vec2(j as f32 / sectors as f32, (i as f32 + tree.trunk_segments as f32) / (tree.trunk_segments + tree.branch_segments) as f32));
                }
            }

            // Generate indices for branches
            let offset = ((tree.trunk_segments + 1) * (sectors + 1) + (branch_count - 1) * (tree.branch_segments + 1) * (sectors + 1)) as u32;
            for i in 0..tree.branch_segments {
                for j in 0..sectors {
                    let k1 = (i * (sectors + 1) + j + offset) as u32;
                    let k2 = ((i + 1) * (sectors + 1) + j + offset) as u32;

                    indices.push(k1);
                    indices.push(k1 + 1);
                    indices.push(k2);

                    indices.push(k1 + 1);
                    indices.push(k2 + 1);
                    indices.push(k2);
                }
            }
        }

        // Generate branches
        let mut branch_bases = top_vertices.clone();
        for _ in 0..branch_count {
            let base_index = rng.gen_range(0..branch_bases.len());
            let branch_base = branch_bases[base_index];
            branch_bases.remove(base_index);

            let mut branch_direction = vec3(
                rng.gen::<f32>() - 0.5,
                rng.gen::<f32>() - 0.5,
                rng.gen::<f32>() - 0.5
            ).normalize();

            for i in (0..=tree.branch_segments).rev() {
                let variance = rng.gen::<f32>() * radius_variance;
                let z = tree.branch_length * (i as f32 / tree.branch_segments as f32);
                let radius = branch_radius * (1.0 - i as f32 / 2.0 / tree.branch_segments as f32) * (1.0 - variance);

                // Add some randomness to the direction of the branch
                let random_direction = vec3(
                    rng.gen::<f32>() - 0.5,
                    rng.gen::<f32>() - 0.5,
                    rng.gen::<f32>() - 0.5
                ).normalize() * direction_variance;
                branch_direction = (branch_direction + random_direction).normalize();

                let branch_position = branch_base + branch_direction * z;

                for j in 0..=sectors {
                    let sector_angle = j as f32 * sector_step;
                    let x = radius * sector_angle.cos();
                    let y = radius * sector_angle.sin();

                    vertices.push(branch_position + vec3(x, y, 0.0));
                    normals.push(vec3(x, y, 0.0).normalize());
                    uvs.push(vec2(j as f32 / sectors as f32, (i as f32 + tree.trunk_segments as f32) / (tree.trunk_segments + tree.branch_segments) as f32));
                }
            }

            // Generate indices for branches
            let offset = ((tree.trunk_segments + 1) * (sectors + 1) + (branch_count - 1) * (tree.branch_segments + 1) * (sectors + 1)) as u32;
            for i in 0..tree.branch_segments {
                for j in 0..sectors {
                    let k1 = (i * (sectors + 1) + j + offset) as u32;
                    let k2 = ((i + 1) * (sectors + 1) + j + offset) as u32;

                    indices.push(k1);
                    indices.push(k1 + 1);
                    indices.push(k2);

                    indices.push(k1 + 1);
                    indices.push(k2 + 1);
                    indices.push(k2);
                }
            }
        }

        // Generate foliage
        let foliage_count = tree.foliage_density + tree.foliage_segments;
        let foliage_radius_variance = 2.0;
        let foliage_position_variance = vec3(6.0, 6.0, 6.0);

        for _ in 0..foliage_count {
            let foliage_radius = tree.foliage_radius * (1.0 - rng.gen::<f32>() * foliage_radius_variance);
            let foliage_position = top_vertices[rng.gen_range(0..top_vertices.len())] + vec3(
                rng.gen::<f32>() * foliage_position_variance.x,
                rng.gen::<f32>() * foliage_position_variance.y,
                rng.gen::<f32>() * foliage_position_variance.z + 2.0,
            );

            let segments = tree.foliage_segments; // Control the number of segments for each foliage sphere
            let density = tree.foliage_density; // Control the density of vertices around each sphere
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
                    sphere_normals.push(vec3(x - foliage_position.x, y - foliage_position.y, z - foliage_position.z).normalize());
                    sphere_uvs.push(vec2(j as f32 / density as f32, i as f32 / segments as f32));
                }
            }

            let sphere_indices = generate_sphere_indices(segments as usize, density as usize);

            vertices.extend(sphere_vertices.clone());
            normals.extend(sphere_normals);
            uvs.extend(sphere_uvs);

            let offset = vertices.len() - sphere_vertices.len();
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

        let mut mesh = Mesh {
            positions: vertices,
            texcoords: vec![uvs],
            normals: Some(normals),
            indices: indices,
            ..Default::default()
        };
        mesh.name = "tree".to_string();
        //mesh.create_tangents();
        mesh
    }
}
