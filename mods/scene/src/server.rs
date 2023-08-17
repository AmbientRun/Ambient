use ambient_api::{
    core::{
        app::components::main_scene,
        physics::components::{cube_collider, dynamic, physics_controlled, plane_collider},
        // prefab::components::prefab_from_url,
        primitives::{
            components::{cube, quad},
            concepts::make_sphere,
        },
        rendering::components::{cast_shadows, color, fog_density, light_diffuse, sky, sun},
        transform::{
            components::{rotation, scale, translation},
            concepts::make_transformable,
        },
    },
    prelude::*,
};

use embers::afps_schema::components::player_team;

#[main]
pub async fn main() {
    Entity::new()
        .with(quad(), ())
        .with(plane_collider(), ())
        .with(scale(), Vec3::ONE * 1000.)
        .spawn();
    Entity::new()
        .with_merge(make_transformable())
        .with(sky(), ())
        .spawn();
    // Entity::new()
    //     .with(translation(), vec3(10., 10., 0.))
    //     .with(cube(), ())
    //     .with(cube_collider(), vec3(1., 1., 1.))
    //     .with(physics_controlled(), ())
    //     .with(dynamic(), true)
    //     .spawn();
    Entity::new()
        .with_merge(make_transformable())
        .with(sun(), Default::default())
        .with(rotation(), Quat::from_rotation_y(-0.6))
        .with(main_scene(), ())
        .with(light_diffuse(), Vec3::ONE)
        .with(fog_density(), 0.1)
        .spawn();

    for _ in 0..40 {
        let pos = random::<Vec2>() * 100. - 50.;
        Entity::new()
            .with_merge(make_sphere())
            .with(cast_shadows(), ())
            .with(cube(), ())
            .with(translation(), vec3(pos.x, pos.y, 0.0))
            .with(color(), vec4(0.9, 0.9, 0.9, 1.))
            .with(cube_collider(), Vec3::ONE)
            .with(
                scale(),
                vec3(
                    random::<f32>() * 5. + 0.5,
                    random::<f32>() * 6. + 0.5,
                    random::<f32>() * 7. + 0.5,
                ),
            )
            .with(physics_controlled(), ())
            .with(dynamic(), true)
            .spawn();
    }

    for _ in 0..60 {
        let pos = random::<Vec2>() * 100. - 50.;
        Entity::new()
            .with_merge(make_sphere())
            .with(cast_shadows(), ())
            .with(cube(), ())
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
            .with(physics_controlled(), ())
            .with(dynamic(), true)
            .with(player_team(), 0)
            .spawn();
    }
}
