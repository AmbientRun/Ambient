use ambient_api::{
    client::{material, mesh, sampler, texture},
    components::core::{
        camera::aspect_ratio_from_window,
        primitives::quad,
        procedurals::{procedural_material, procedural_mesh},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    mesh::Vertex,
    prelude::*,
};

use components::rotating_sun;
use glam::*;
use noise::{utils::*, Fbm, Perlin};
use palette::IntoColor;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

const TAU: f32 = std::f32::consts::TAU;
const RESOLUTION_X: u32 = 32;
const RESOLUTION_Y: u32 = 8;
const TEXTURE_RESOLUTION_X: u32 = 4 * RESOLUTION_X;
const TEXTURE_RESOLUTION_Y: u32 = 4 * RESOLUTION_Y;
const SIZE_X: f32 = RESOLUTION_X as f32 / RESOLUTION_Y as f32;
const SIZE_Y: f32 = 1.0;
const WAVE_AMPLITUDE: f32 = 0.25;
const WAVE_FREQUENCY: f32 = 0.5 * TAU;

#[derive(Clone)]
pub struct TreeMesh {
    pub seed: i32,
    pub trunk_radius: f32,
    pub trunk_height: f32,
    pub trunk_segments: u32,
    pub foliage_radius: f32,
    pub foliage_density: u32,
    pub foliage_segments: u32,
}

impl Default for TreeMesh {
    fn default() -> Self {
        Self {
            seed: 123 as i32,
            trunk_radius: 3.0,
            trunk_height: 15.0,
            trunk_segments: 8,
            foliage_radius: 2.0,
            foliage_density: 5,
            foliage_segments: 5,
        }
    }
}
pub struct MeshDescriptor {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

pub fn create_tree(
    t: TreeMesh,
    seed: i32,
    trunk_radius: f32,
    trunk_height: f32,
    trunk_segments: u32,
) -> MeshDescriptor {
    let mut tree = t;

    tree.trunk_radius = trunk_radius;
    tree.trunk_height = trunk_height;
    tree.trunk_segments = trunk_segments;
    tree.seed = seed;

    // Create the trunk
    let (mut vertices1, top_vertices1, mut normals1, mut uvs1) = build_trunk(&tree);

    let sectors = 12;
    let trunk_segments = tree.trunk_segments;
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
    let foliage_position_variance = vec3(5.0, 5.0, 3.0);

    for i in 0..foliage_count {
        let foliage_radius = tree.foliage_radius
            * (1.0 - gen_rn(tree.seed + i as i32, 0.0, 1.0) * foliage_radius_variance);
        let foliage_position = top_vertices1
            [gen_rn(tree.seed, 0.0, top_vertices1.len() as f32) as usize]
            + vec3(
                gen_rn(tree.seed + i as i32, 0.0, 1.0) * foliage_position_variance.x,
                gen_rn(tree.seed + i as i32 + 1, 0.0, 1.0) * foliage_position_variance.y,
                gen_rn(tree.seed + i as i32 + 2, 0.0, 1.0) * foliage_position_variance.z,
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

    let mut vec_of_vertex: Vec<Vertex> = Vec::with_capacity(vertices1.len());

    for i in 0..vertices1.len() {
        let px = vertices1[i].x;
        let py = vertices1[i].y;
        let pz = vertices1[i].z;
        let u = uvs1[i].x;
        let v = uvs1[i].y;
        let nx = normals1[i].x;
        let ny = normals1[i].y;
        let nz = normals1[i].z;

        let v = mesh::Vertex {
            position: vec3(px, py, pz) + vec3(-0.5 * SIZE_X, -0.5 * SIZE_Y, 0.0),
            normal: vec3(nx, ny, nz),
            tangent: vec3(1.0, 0.0, 0.0),
            texcoord0: vec2(u, v),
        };
        vec_of_vertex.push(v);
    }

    let vcs = vec_of_vertex.clone();
    let ids = indices.clone();

    MeshDescriptor {
        vertices: vec_of_vertex,
        indices: indices.clone(),
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

fn make_camera() {
    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with_default(main_scene())
        .with(translation(), vec3(10.0, 10.0, 4.0) * 2.0)
        .with(lookat_target(), vec3(0.0, 3.0, 0.0))
        .spawn();
}

#[element_component]
fn App(_hooks: &mut Hooks, sun_id: EntityId) -> Element {
    FocusRoot::el([FlowColumn::el([FlowRow::el([Button::new(
        "Toggle sun rotation",
        move |_| {
            entity::mutate_component(sun_id, rotating_sun(), |rotating_sun| {
                *rotating_sun = !*rotating_sun;
            });
        },
    )
    .el()])])
    .with_padding_even(10.0)])
}

fn make_lighting() {
    let sun_id = Entity::new()
        .with_merge(make_transformable())
        .with_default(sun())
        .with(
            rotation(),
            Quat::from_rotation_y(-45_f32.to_radians())
                * Quat::from_rotation_z(-45_f32.to_radians()),
        )
        .with(light_diffuse(), Vec3::ONE * 4.0)
        .with_default(main_scene())
        .with(rotating_sun(), false)
        .spawn();
    App::el(sun_id).spawn_interactive();
    query((rotation(), (rotating_sun())))
        .requires(sun())
        .each_frame(move |suns| {
            for (sun_id, (sun_rotation, rotating_sun)) in suns {
                if !rotating_sun {
                    continue;
                }
                entity::set_component(
                    sun_id,
                    rotation(),
                    Quat::from_rotation_z(frametime()) * sun_rotation,
                );
            }
        });
}

fn make_ground() {
    Entity::new()
        .with_merge(make_transformable())
        .with_default(quad())
        .with(color(), vec4(0.25, 1.0, 0.25, 1.0))
        .with(translation(), vec3(0.0, 0.0, -0.5))
        .with(scale(), 32.0 * Vec3::ONE)
        .spawn();
}

fn make_texture<PixelFn>(mut pixel_fn: PixelFn) -> ProceduralTextureHandle
where
    PixelFn: FnMut(f32, f32) -> [u8; 4],
{
    let mut pixels = vec![0_u8; (4 * TEXTURE_RESOLUTION_X * TEXTURE_RESOLUTION_Y) as usize];
    for y in 0..TEXTURE_RESOLUTION_Y {
        for x in 0..TEXTURE_RESOLUTION_X {
            let dst = (4 * (x + y * TEXTURE_RESOLUTION_X)) as usize;
            let dst = &mut pixels[dst..(dst + 4)];
            let px = (x as f32 + 0.5) / (TEXTURE_RESOLUTION_X as f32);
            let py = (y as f32 + 0.5) / (TEXTURE_RESOLUTION_Y as f32);
            dst.copy_from_slice(&pixel_fn(px, py));
        }
    }
    texture::create_2d(&texture::Descriptor2D {
        width: TEXTURE_RESOLUTION_X,
        height: TEXTURE_RESOLUTION_Y,
        format: texture::Format::Rgba8Unorm,
        data: &pixels,
    })
}

fn default_base_color(_: f32, _: f32) -> [u8; 4] {
    [255, 255, 255, 255]
}

fn default_normal(_: f32, _: f32) -> [u8; 4] {
    [128, 128, 255, 0]
}

fn default_metallic_roughness(_: f32, _: f32) -> [u8; 4] {
    [255, 255, 0, 0]
}

fn default_nearest_sampler() -> ProceduralSamplerHandle {
    sampler::create(&sampler::Descriptor {
        address_mode_u: sampler::AddressMode::ClampToEdge,
        address_mode_v: sampler::AddressMode::ClampToEdge,
        address_mode_w: sampler::AddressMode::ClampToEdge,
        mag_filter: sampler::FilterMode::Nearest,
        min_filter: sampler::FilterMode::Nearest,
        mipmap_filter: sampler::FilterMode::Nearest,
    })
}

fn default_linear_sampler() -> ProceduralSamplerHandle {
    sampler::create(&sampler::Descriptor {
        address_mode_u: sampler::AddressMode::ClampToEdge,
        address_mode_v: sampler::AddressMode::ClampToEdge,
        address_mode_w: sampler::AddressMode::ClampToEdge,
        mag_filter: sampler::FilterMode::Linear,
        min_filter: sampler::FilterMode::Linear,
        mipmap_filter: sampler::FilterMode::Linear,
    })
}

fn make_procedural<BaseColorFn, NormalFn, MetallicRoughnessFn, SamplerFn>(
    world_translation: Vec3,
    base_color_fn: BaseColorFn,
    normal_fn: NormalFn,
    metallic_roughness_fn: MetallicRoughnessFn,
    sampler_fn: SamplerFn,
    transparent: bool,
) where
    BaseColorFn: FnMut(f32, f32) -> [u8; 4],
    NormalFn: FnMut(f32, f32) -> [u8; 4],
    MetallicRoughnessFn: FnMut(f32, f32) -> [u8; 4],
    SamplerFn: FnOnce() -> ProceduralSamplerHandle,
{
    let mut vertices = vec![];
    let mut indices = vec![];
    for y in 0..=RESOLUTION_Y {
        for x in 0..=RESOLUTION_X {
            let px = SIZE_X * (x as f32) / (RESOLUTION_X as f32);
            let py = SIZE_Y * (y as f32) / (RESOLUTION_Y as f32);
            let pz = WAVE_AMPLITUDE * f32::sin(WAVE_FREQUENCY * px);
            let u = (x as f32) / (RESOLUTION_X as f32);
            let v = (y as f32) / (RESOLUTION_Y as f32);
            vertices.push(mesh::Vertex {
                position: vec3(px, py, pz) + vec3(-0.5 * SIZE_X, -0.5 * SIZE_Y, 0.0),
                normal: vec3(0.0, 0.0, 1.0),
                tangent: vec3(1.0, 0.0, 0.0),
                texcoord0: vec2(u, v),
            });
        }
    }
    for y in 0..RESOLUTION_Y {
        for x in 0..RESOLUTION_X {
            let i0 = x + y * (RESOLUTION_X + 1);
            let i1 = (x + 1) + y * (RESOLUTION_X + 1);
            let i2 = x + (y + 1) * (RESOLUTION_X + 1);
            let i3 = (x + 1) + (y + 1) * (RESOLUTION_X + 1);
            indices.extend([i0, i1, i2]);
            indices.extend([i1, i3, i2]);
        }
    }
    for triangle in indices.chunks_exact(3) {
        let [i0, i1, i2]: [_; 3] = triangle.try_into().unwrap();
        let p0 = vertices[i0 as usize].position;
        let p1 = vertices[i1 as usize].position;
        let p2 = vertices[i2 as usize].position;
        let n01 = (p1 - p0).normalize();
        let n02 = (p2 - p0).normalize();
        let n = n01.cross(n02).normalize();
        vertices[i0 as usize].normal += n;
        vertices[i1 as usize].normal += n;
        vertices[i2 as usize].normal += n;
    }
    for vertex in &mut vertices {
        vertex.normal = vertex.normal.normalize();
    }

    let seed = 12345;
    let num_trees = 15;

    let base_color_map = make_texture(base_color_fn);
    let normal_map = make_texture(normal_fn);
    let metallic_roughness_map = make_texture(metallic_roughness_fn);
    let sampler = sampler_fn();
    let material = material::create(&material::Descriptor {
        base_color_map,
        normal_map,
        metallic_roughness_map,
        sampler,
        transparent,
    });

    // lets plant some trees :)
    for i in 0..num_trees {
        let trunk_radius = gen_rn(seed + i, 2.0, 3.0);
        let trunk_height = gen_rn(seed + i, 15.0, 20.0);
        let trunk_segments = gen_rn(seed + i, 6.0, 12.0) as u32;

        let tree = create_tree(
            TreeMesh::default(),
            seed,
            trunk_radius,
            trunk_height,
            trunk_segments,
        );

        let td = mesh::Descriptor {
            vertices: &tree.vertices,
            indices: &tree.indices,
        };

        let mesh = mesh::create(&td);

        Entity::new()
            .with_merge(make_transformable())
            .with(procedural_mesh(), mesh)
            .with(procedural_material(), material)
            .with_default(cast_shadows())
            .with(scale(), Vec3::ONE * gen_rn(i, 0.2, 0.4))
            .with(
                translation(),
                vec3(
                    gen_rn(seed + i, 0.0, 15.0),
                    gen_rn(seed + seed + i, 0.0, 15.0),
                    0.0,
                ),
            )
            .spawn();
    }
}

fn make_procedurals() {
    const X: f32 = 2.125;
    const Y: f32 = 1.25;

    let rng = rand_pcg::Pcg64::seed_from_u64(0);
    let dist_zero_to_255 = rand::distributions::Uniform::new_inclusive(0_u8, 255_u8);
    let dist_minus_one_to_one = rand::distributions::Uniform::new_inclusive(-1.0_f32, 1.0_f32);

    // Interpolated hue.
    make_procedural(
        vec3(-X, Y, 0.0),
        |x, _| {
            let hsl = palette::Hsl::new(360.0 * x, 1.0, 0.5).into_format::<f32>();
            let rgb: palette::LinSrgb = hsl.into_color();
            let r = (255.0 * rgb.red) as u8;
            let g = (255.0 * rgb.green) as u8;
            let b = (255.0 * rgb.blue) as u8;
            let a = 255;
            [r, g, b, a]
        },
        default_normal,
        default_metallic_roughness,
        default_nearest_sampler,
        false,
    );
}

#[main]
pub async fn main() {
    make_camera();
    make_lighting();
    make_ground();
    make_procedurals();
}
