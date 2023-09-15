use ambient_api::{
    core::{
        app::components::main_scene,
        physics::components::{cube_collider, dynamic, physics_controlled, plane_collider},
        primitives::components::cube,
        rendering::components::{
            cast_shadows, color, fog_color, fog_density, fog_height_falloff, light_diffuse, sky,
            sun, water,
        },
        transform::components::{rotation, scale, translation},
    },
    prelude::*,
};

#[main]
pub async fn main() {
    // Make sky
    Entity::new().with(sky(), ()).spawn();

    // Make sun
    Entity::new()
        .with(sun(), 0.0)
        .with(rotation(), Default::default())
        .with(main_scene(), ())
        .with(light_diffuse(), Vec3::ONE)
        .with(fog_color(), vec3(0.88, 0.37, 0.34))
        .with(fog_density(), 0.01)
        .with(fog_height_falloff(), 0.1)
        .with(rotation(), Quat::from_rotation_y(190.0f32.to_radians()))
        .spawn();

    // Make water
    Entity::new()
        .with(water(), ())
        .with(physics_controlled(), ())
        .with(plane_collider(), ())
        .with(dynamic(), false)
        .with(scale(), Vec3::ONE * 4000.)
        .spawn();

    for _ in 0..40 {
        let pos = random::<Vec2>() * 100. - 50.;
        Entity::new()
            .with(cast_shadows(), ())
            .with(cube(), ())
            .with(translation(), vec3(pos.x, pos.y, 2.))
            .with(color(), random::<Vec3>().extend(1.))
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
}
