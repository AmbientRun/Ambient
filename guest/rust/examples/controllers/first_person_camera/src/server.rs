use ambient_api::{
    core::{
        physics::components::{plane_collider, sphere_collider},
        player::components::is_player,
        primitives::{components::quad, concepts::Sphere},
        rendering::components::color,
        transform::components::{scale, translation},
    },
    prelude::*,
};
use packages::fps_controller::components::{camera_distance, use_fps_controller};

#[main]
pub fn main() {
    Entity::new()
        .with(quad(), ())
        .with(scale(), Vec3::ONE * 10.)
        .with(color(), vec4(1., 0., 0., 1.))
        .with(plane_collider(), ())
        .spawn();

    Entity::new()
        .with_merge(Sphere {
            sphere: (),
            sphere_radius: 0.5,
            sphere_sectors: 36,
            sphere_stacks: 18,
        })
        .with(sphere_collider(), 0.5)
        .with(translation(), vec3(5., 5., 1.))
        .spawn();

    spawn_query(is_player()).bind(move |players| {
        for (id, _) in players {
            entity::add_components(
                id,
                Entity::new()
                    .with(use_fps_controller(), ())
                    .with(camera_distance(), 0.0),
            );
        }
    });
}
