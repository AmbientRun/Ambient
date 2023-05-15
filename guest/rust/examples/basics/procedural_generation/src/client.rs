use ambient_api::client::{material, mesh, texture};
use ambient_api::components::core::camera::aspect_ratio_from_window;
use ambient_api::components::core::primitives::cube;
use ambient_api::concepts::{make_perspective_infinite_reverse_camera, make_transformable};
use ambient_api::prelude::*;

// Todo: cleanup.

#[main]
pub async fn main() {
    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with_default(main_scene())
        .with(translation(), vec3(2.0, 3.0, 2.0 * 4.0))
        .with(lookat_target(), vec3(0.0, 0.0, 0.0))
        .spawn();

    let mut state = rand_pcg::Pcg64::seed_from_u64(0);
    let dist = rand::distributions::Uniform::new_inclusive(0.0, 1.0);

    let resolution: u32 = 32;
    let mut temp_vertices = vec![];
    let size = 8.0;
    let half_size = 0.5 * size;
    for y in 0..=resolution {
        for x in 0..=resolution {
            let x = (x as f32) / (resolution as f32);
            let y = (y as f32) / (resolution as f32);
            let z = 0.0; //0.25 * dist.sample(&mut state);
            let position = vec3(size * x - half_size, size * y - half_size, z);
            temp_vertices.push(mesh::Vertex {
                position,
                normal: vec3(0.0, 0.0, 1.0),
                tangent: vec3(1.0, 0.0, 0.0),
                texcoord0: vec2(x, y),
            });
        }
    }
    let mut vertices = vec![];
    let mut indices = vec![];
    for y in 0..resolution {
        for x in 0..resolution {
            let i0 = x + y * (resolution + 1);
            let i1 = (x + 1) + y * (resolution + 1);
            let i2 = x + (y + 1) * (resolution + 1);
            let i3 = (x + 1) + (y + 1) * (resolution + 1);
            let mut v0 = temp_vertices[i0 as usize];
            let mut v1 = temp_vertices[i1 as usize];
            let mut v2 = temp_vertices[i2 as usize];
            let mut v3 = temp_vertices[i3 as usize];
            let n10 = (v1.position - v0.position).normalize();
            let n20 = (v2.position - v0.position).normalize();
            let n = n10.cross(n20).normalize();
            v0.normal = n;
            v1.normal = n;
            v2.normal = n;
            v3.normal = n;
            let i = vertices.len() as u32;
            vertices.extend([v0, v1, v2, v3]);
            indices.extend([i, i + 1, i + 2]);
            indices.extend([i + 1, i + 3, i + 2]);
        }
    }
    let mesh_url = mesh::create(&vertices, &indices);

    let texture_width = 64;
    let texture_height = 64;
    let mut texture_data = Vec::with_capacity((4 * texture_width * texture_height) as usize);
    {
        use noise::{utils::*, Fbm, Perlin};

        let fbm = Fbm::<Perlin>::default();
        let bounds = 5.0;
        let noise_map = PlaneMapBuilder::<_, 2>::new(fbm)
            .set_size(texture_width as _, texture_height as _)
            .set_x_bounds(-bounds, bounds)
            .set_y_bounds(-bounds, bounds)
            .build();
        for v in noise_map {
            let v = ((v * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0) as u8;
            texture_data.extend([v, 0, 0, 255]);
        }
    }

    let base_color_map_url = texture::create_2d(
        texture_width,
        texture_height,
        texture::Format::Rgba8Unorm,
        &texture_data,
    );

    let normal_map_url = texture::create_2d(1, 1, texture::Format::Rgba8Unorm, &[128, 128, 255, 0]);
    let metallic_roughness_map_url =
        texture::create_2d(1, 1, texture::Format::Rgba8Unorm, &[255, 255, 255, 255]);

    let material_desc = material::Descriptor {
        base_color_map: &base_color_map_url,
        normal_map: &normal_map_url,
        metallic_roughness_map: &metallic_roughness_map_url,
    };
    let material_url = material::create(&material_desc);

    Entity::new()
        .with(mesh_from_url(), mesh_url)
        .with(material_from_url(), material_url)
        .with(color(), vec4(1.0, 1.0, 1.0, 1.0))
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with_default(cube())
        .with(translation(), vec3(0.0, 0.0, -2.0))
        .with(scale(), Vec3::ONE)
        .with(color(), Vec4::ONE)
        .spawn();

    // Sun.
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

    // Sun rotation.
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
