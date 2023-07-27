use ambient_api::{
    components::core::{
        app::main_scene,
        physics::{cube_collider, dynamic, physics_controlled, plane_collider},
        prefab::prefab_from_url,
        primitives::{cube, quad},
        rendering::{fog_density, light_diffuse, sky, sun},
        transform::{rotation, scale, translation},
    },
    concepts::make_transformable,
    prelude::*,
};

use ambient_api::{
    components::core::{
        camera::aspect_ratio_from_window,
        physics::{angular_velocity, linear_velocity, sphere_collider},
        primitives::sphere_radius,
        rendering::{cast_shadows, color, water},
        transform::lookat_target,
    },
    concepts::{make_perspective_infinite_reverse_camera, make_sphere},
    prelude::*,
};

#[main]
pub async fn main() {
    Entity::new()
        .with_default(quad())
        .with_default(plane_collider())
        .with(scale(), Vec3::ONE * 1000.)
        .spawn();
    Entity::new()
        .with_merge(make_transformable())
        .with_default(sky())
        .spawn();
    Entity::new()
        .with(translation(), vec3(10., 10., 0.))
        .with_default(cube())
        .with(cube_collider(), vec3(1., 1., 1.))
        .with_default(physics_controlled())
        .with(dynamic(), true)
        .spawn();
    Entity::new()
        .with_merge(make_transformable())
        .with_default(sun())
        .with(rotation(), Quat::from_rotation_y(-0.6))
        .with_default(main_scene())
        .with(light_diffuse(), Vec3::ONE)
        .with(fog_density(), 0.1)
        .spawn();

    for _ in 0..100 {
        let pos = random::<Vec2>() * 100. - 50.;
        Entity::new()
            .with_merge(make_sphere())
            .with_default(cast_shadows())
            .with_default(cube())
            .with(translation(), vec3(pos.x, pos.y, 0.0))
            .with(
                color(),
                vec4(random::<f32>(), random::<f32>(), random::<f32>(), 1.),
            )
            .with(cube_collider(), Vec3::ONE)
            .with(
                scale(),
                vec3(
                    random::<f32>() * 5. + 0.5,
                    random::<f32>() * 6. + 0.5,
                    random::<f32>() * 7. + 0.5,
                ),
            )
            .with_default(physics_controlled())
            .with(dynamic(), true)
            .with(components::player_team(), 0)
            .spawn();
    }
}
