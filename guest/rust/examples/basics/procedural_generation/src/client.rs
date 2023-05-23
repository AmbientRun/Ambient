use ambient_api::{
    client::{material, mesh, sampler, texture},
    components::core::{
        camera::aspect_ratio_from_window,
        primitives::{cube, quad, sphere_radius},
        procedurals::{procedural_material, procedural_mesh},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_sphere, make_transformable},
    prelude::*,
};
use noise::{utils::*, Fbm, Perlin};
use palette::IntoColor;

const TAU: f32 = std::f32::consts::TAU;
const RESOLUTION_X: u32 = 32;
const RESOLUTION_Y: u32 = 8;
const TEXTURE_RESOLUTION_X: u32 = 4 * RESOLUTION_X;
const TEXTURE_RESOLUTION_Y: u32 = 4 * RESOLUTION_Y;
const SIZE_X: f32 = RESOLUTION_X as f32 / RESOLUTION_Y as f32;
const SIZE_Y: f32 = 1.0;
const WAVE_AMPLITUDE: f32 = 0.25;
const WAVE_FREQUENCY: f32 = 0.5 * TAU;
const ROTATING_SUN: bool = false;

fn make_camera() {
    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with_default(main_scene())
        .with(translation(), vec3(0.0, 3.0, 4.0) * 2.0)
        .with(lookat_target(), vec3(0.0, 3.0, 0.0))
        .spawn();
}

fn make_lighting() {
    Entity::new()
        .with_merge(make_transformable())
        .with_default(sun())
        .with(
            rotation(),
            Quat::from_rotation_y(-45_f32.to_radians())
                * Quat::from_rotation_z(-45_f32.to_radians()),
        )
        .with(light_diffuse(), Vec3::ONE * 4.0)
        .with_default(main_scene())
        .spawn();
    if ROTATING_SUN {
        query(rotation()).requires(sun()).each_frame(move |suns| {
            for (sun_id, sun_rotation) in suns {
                entity::set_component(
                    sun_id,
                    rotation(),
                    Quat::from_rotation_z(frametime()) * sun_rotation,
                );
            }
        });
    }
}

fn make_coordinate_system() {
    let make_cube = |t: Vec3, s: Vec3, c: Vec4| {
        Entity::new()
            .with_merge(make_transformable())
            .with_default(cube())
            .with(translation(), t)
            .with(scale(), s)
            .with(color(), c)
            .with_default(main_scene())
            .spawn();
    };
    let make_sphere = |t: Vec3, r: f32, c: Vec4| {
        Entity::new()
            .with_merge(make_transformable())
            .with_merge(make_sphere())
            .with(translation(), t)
            .with(sphere_radius(), r)
            .with(color(), c)
            .with_default(main_scene())
            .spawn();
    };
    let make_axis = |axis: Vec3| {
        make_cube(axis, Vec3::ONE * 0.25, axis.extend(1.0));
    };
    make_sphere(Vec3::ZERO, 0.25, Vec4::ONE);
    make_axis(Vec3::X);
    make_axis(Vec3::Y);
    make_axis(Vec3::Z);
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
    let mesh = mesh::create(&mesh::Descriptor {
        vertices: &vertices,
        indices: &indices,
    });
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
    Entity::new()
        .with_merge(make_transformable())
        .with(procedural_mesh(), mesh)
        .with(procedural_material(), material)
        .with(translation(), world_translation)
        .with_default(cast_shadows())
        .spawn();
}

fn make_procedurals() {
    const X: f32 = 2.125;
    const Y: f32 = 1.25;

    let mut rng = rand_pcg::Pcg64::seed_from_u64(0);
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

    // Random base color.
    make_procedural(
        vec3(X, Y, 0.0),
        |_, _| {
            let r = dist_zero_to_255.sample(&mut rng);
            let g = dist_zero_to_255.sample(&mut rng);
            let b = dist_zero_to_255.sample(&mut rng);
            let a = 255;
            [r, g, b, a]
        },
        default_normal,
        default_metallic_roughness,
        default_nearest_sampler,
        false,
    );

    // Random normal on +Z hemisphere.
    make_procedural(
        vec3(-X, 2.0 * Y, 0.0),
        default_base_color,
        |_, _| {
            let mut n = vec3(
                dist_minus_one_to_one.sample(&mut rng),
                dist_minus_one_to_one.sample(&mut rng),
                dist_minus_one_to_one.sample(&mut rng),
            )
            .normalize();
            if n.dot(Vec3::Z) < 0.0 {
                n = -n;
            }
            [
                ((n.x + 1.0) * 255.0) as u8,
                ((n.y + 1.0) * 255.0) as u8,
                ((n.z + 1.0) * 255.0) as u8,
                0,
            ]
        },
        default_metallic_roughness,
        default_nearest_sampler,
        false,
    );

    // Random metallic.
    make_procedural(
        vec3(X, 2.0 * Y, 0.0),
        default_base_color,
        default_normal,
        |_, _| [dist_zero_to_255.sample(&mut rng), 0, 0, 0],
        default_nearest_sampler,
        false,
    );

    // Random roughness.
    make_procedural(
        vec3(-X, 3.0 * Y, 0.0),
        default_base_color,
        default_normal,
        |_, _| [255, dist_zero_to_255.sample(&mut rng), 0, 0],
        default_nearest_sampler,
        false,
    );

    // Perlin noise.
    let fbm = Fbm::<Perlin>::default();
    let bounds = 2.0;
    let noise_map = PlaneMapBuilder::<_, 2>::new(fbm)
        .set_size(TEXTURE_RESOLUTION_X as _, TEXTURE_RESOLUTION_Y as _)
        .set_y_bounds(-bounds * f64::from(SIZE_Y), bounds * f64::from(SIZE_Y))
        .set_x_bounds(-bounds * f64::from(SIZE_X), bounds * f64::from(SIZE_X))
        .build();
    let mut noise_iter = noise_map.iter();
    make_procedural(
        vec3(X, 3.0 * Y, 0.0),
        |_, _| {
            let noise = *noise_iter.next().unwrap();
            let noise = (255.0 * 0.5 * (noise + 1.0)) as u8;
            [noise, noise, noise, 255]
        },
        default_normal,
        default_metallic_roughness,
        default_linear_sampler,
        false,
    );

    // Random alpha.
    make_procedural(
        vec3(-X, 4.0 * Y, 0.0),
        |_, _| {
            let r = 255;
            let g = 255;
            let b = 255;
            let a = dist_zero_to_255.sample(&mut rng);
            [r, g, b, a]
        },
        default_normal,
        default_metallic_roughness,
        default_nearest_sampler,
        true,
    );
}

#[main]
pub async fn main() {
    make_camera();
    make_lighting();
    make_coordinate_system();
    make_ground();
    make_procedurals();
}
