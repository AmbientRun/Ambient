use ambient_api::{
    core::{
        physics::components::{cube_collider, dynamic, physics_controlled},
        primitives::components::cube,
        rendering::components::{cast_shadows, color},
        transform::components::{scale, translation},
    },
    prelude::*,
};

#[main]
pub async fn main() {
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
