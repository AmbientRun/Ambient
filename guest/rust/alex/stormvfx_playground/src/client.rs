use ambient_api::{
    core::{
        primitives::concepts::Sphere,
        transform::components::{scale, translation},
    },
    prelude::*,
};

use packages::this::components::{dropage, dropagerate, dropvel};

#[main]
pub fn main() {
    for _ in 1..100 {
        Sphere::suggested()
            .make()
            .with(
                translation(),
                vec3(
                    random::<f32>() * 100. - 50.,
                    random::<f32>() * 100. - 50.,
                    random::<f32>() * 100. - 50.,
                ),
            )
            .with(scale(), Vec3::ZERO)
            .with(dropvel(), vec3(-1., 0., 0.))
            .with(dropage(), 0.)
            .with(dropagerate(), 1.)
            .spawn();
    }

    query((dropvel(), dropage(), dropagerate())).each_frame(|drops| {
        for (drop, (vel, age, agerate)) in drops {
            entity::set_component(drop, dropage(), age + agerate * delta_time());
            entity::mutate_component(drop, translation(), |pos| *pos = *pos + vel * delta_time());
            entity::set_component(drop, scale(), Vec3::splat((age * 3.14).sin()));
        }
    });
}
