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
        let mut indices = Vec::new();
        let mut rng = rand::thread_rng();

        let branch_count = 1; //rng.gen_range(4..=7);
        let branch_segments = tree.branch_segments;

        let mut trunk_direction = vec3(0.0, 0.0, 1.0);
        let direction_variance = 0.08;

        let radius_variance = 0.02;
        let mut branch_radius = tree.trunk_radius * 0.5;

        let mut top_position = vec3(0.0, 0.0, 0.0);

        // Generate trunk
        for i in 0..=tree.trunk_segments {
            let variance = rng.gen::<f32>() * radius_variance;
            let z = tree.trunk_height * (i as f32 / tree.trunk_segments as f32);
            let radius = tree.trunk_radius * (1.0 - i as f32 / tree.trunk_segments as f32) * (1.0 - variance);

            // Ensure that the top segment of the trunk has a minimum radius of 0.1
            let top_segment_radius = tree.trunk_radius * 0.1;
            let radius = if i == tree.trunk_segments && radius < top_segment_radius {
                top_segment_radius
            } else {
                radius
            };

            // Store current radius of the trunk for later use in the branches
            branch_radius = radius + 1.0;

            // Add some randomness to the direction of the trunk
            let random_direction = vec3(
                rng.gen::<f32>() - 0.5,
                rng.gen::<f32>() - 0.5,
                rng.gen::<f32>() - 0.5
            ).normalize() * direction_variance;
            trunk_direction = (trunk_direction + random_direction).normalize();

            top_position = trunk_direction * z;

            for j in 0..=sectors {
                let sector_angle = j as f32 * sector_step;
                let x = radius * sector_angle.cos();
                let y = radius * sector_angle.sin();

            vertices.push(top_position + vec3(x, y, 0.0));
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
    for _ in 0..branch_count {
        let segment_index = tree.trunk_segments;
        let branch_start_index = (segment_index * (sectors + 1)) as usize;
        let branch_end_index = branch_start_index + sectors as usize;

        let branch_start_position = top_position;
        let branch_end_position = vertices[branch_end_index];
        // Add some randomness to the direction of the trunk
        let random_direction = vec3(
            rng.gen::<f32>() - 0.5,
            rng.gen::<f32>() - 0.5,
            rng.gen::<f32>() - 0.5
        ).normalize() * direction_variance * 4.0;
        let branch_direction = ( trunk_direction + random_direction).normalize();//-(branch_end_position - branch_start_position).normalize();
        let branch_length = tree.branch_length;

        let branch_height = branch_length / branch_segments as f32;

        let mut current_branch_position = branch_start_position;

        let mut current_branch_direction = branch_direction;
        //branch_direction;
        let mut branch_radius = branch_radius * 0.5;

        for _ in 0..branch_segments {
            // Add some randomness to the direction of the trunk
            let random_direction = vec3(
                rng.gen::<f32>() - 0.5,
                rng.gen::<f32>() - 0.5,
                rng.gen::<f32>() - 0.5
            ).normalize() * direction_variance * 4.0;
            current_branch_direction = (current_branch_direction + random_direction).normalize();
            let branch_position = current_branch_position + current_branch_direction * branch_height;

            for j in 0..=sectors {
                let sector_angle = j as f32 * sector_step;
                let x = branch_radius * sector_angle.cos();
                let y = branch_radius * sector_angle.sin();

                vertices.push(branch_position + vec3(x, y, 0.0));
                normals.push(vec3(x, y, 0.0).normalize());
                uvs.push(vec2(j as f32 / sectors as f32, 0.0));
            }

            branch_radius *= 0.5 + 0.1;

            let branch_start_offset = branch_start_index as u32;
            let branch_end_offset = (vertices.len() as u32 - (sectors + 1)) as u32;

            for j in 0..sectors {
                let k1 = branch_start_offset + j as u32;
                let k2 = branch_start_offset + (j + 1) as u32;
                let k3 = branch_end_offset + j as u32;
                let k4 = branch_end_offset + (j + 1) as u32;

                indices.push(k1);
                indices.push(k2);
                indices.push(k3);

                indices.push(k2);
                indices.push(k4);
                indices.push(k3);
            }

            current_branch_position = branch_position;

            // Calculate the new branch direction with slight randomness
            let random_direction = vec3(
                rng.gen_range(-0.01..=0.01),
                rng.gen_range(-0.01..=0.01),
                0.02//rng.gen_range(-0.1..=0.1)
            ).normalize();

            current_branch_direction = (current_branch_direction + random_direction).normalize();
        }
    }

    // Combine trunk and branches
    vertices.extend(top_vertices.clone());
    indices.extend(indices.clone().iter().map(|i| *i + top_vertices.len() as u32));

    let mut mesh = Mesh {
        positions: vertices,
        texcoords: vec![uvs],
        normals: Some(normals),
        indices: indices,
        ..Default::default()
    };

    mesh.name = "tree".to_string();
    mesh
}
}