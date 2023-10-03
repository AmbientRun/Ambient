use std::f32::consts::PI;

use ambient_api::{
    core::{
        model::components::model_from_url,
        primitives::components::cube,
        rendering::components::{color, transparency_group},
        transform::components::{rotation, scale, translation},
    },
    prelude::*,
};

use packages::{droplet::components::*, this::components::*};

#[main]
pub fn main() {
    fires(false);
    // spawn_test_camera();
}

fn rr(a: f32, b: f32) -> f32 {
    random::<f32>() * (b - a) + a
}
fn rsplat(nab: f32) -> f32 {
    -nab + random::<f32>() * (nab + nab)
}
fn warp(t: f32, mid1: f32, mid2: f32) -> f32 {
    if t < mid1 {
        t / mid1 * mid2
    } else {
        (t - mid1) / (1. - mid1) * (1. - mid2) + mid2
    }
}
fn triwarp(t: f32, mid: f32) -> f32 {
    if t < mid {
        t / mid
    } else {
        1. - (t - mid) / (1. - mid)
    }
}
fn sinowarp(t: f32, mid: f32) -> f32 {
    (warp(t, mid, 0.5) * PI).sin()
}
fn longsinowarp(t: f32, minmid: f32, maxmid: f32) -> f32 {
    if t < minmid {
        sinowarp(t, minmid)
    } else if t > maxmid {
        sinowarp(t, maxmid)
    } else {
        1.
    }
}

fn fires(test: bool) {
    spawn_query(translation())
        .requires((is_droplet_spawner(), spawns_fire()))
        .bind(|fires| {
            for (fire, place) in fires {
                // create some fires
                for _ in 0..70 {
                    let (hoff, height) = (vec3(rsplat(1.), rsplat(1.), 0.), rr(2., 4.));
                    entity::add_child(
                        fire,
                        Entity::new()
                            .with(tween_atob_a(), place + hoff)
                            .with(tween_atob_b(), place + hoff + vec3(0., 0., height))
                            // .with(tween_sinescale(), 10.0)
                            .with(is_firelick(), ())
                            .with(loop_length(), rr(2.99, 4.99))
                            .with(age(), random::<f32>())
                            .with(
                                model_from_url(),
                                packages::this::assets::url("mt-fire-lick.fbx"),
                            )
                            .with(rotation(), Quat::from_rotation_x(PI))
                            .spawn(),
                    );
                }

                // create like ten smokes
                for _ in 0..10 {
                    let (hoff, height) = (vec3(rsplat(1.), rsplat(1.), 0.), rr(15., 30.));
                    entity::add_child(
                        fire,
                        Entity::new()
                            .with(tween_atob_a(), place + hoff)
                            .with(tween_atob_b(), place + hoff + vec3(0., 0., height))
                            // .with(tween_sinescale(), 10.0)
                            .with(transparency_group(), 1)
                            .with(is_firesmoke(), ())
                            .with(loop_length(), rr(5., 10.))
                            .with(age(), random::<f32>())
                            .with(just_looped(), 1)
                            .with(cube(), ())
                            .spawn(),
                    );
                }
            }
        });

    change_query(age())
        .track_change(age())
        .requires(is_firesmoke())
        .bind(|smokes| {
            for (smoke, age) in smokes {
                entity::add_components(
                    smoke,
                    Entity::new()
                        .with(color(), Vec3::ONE.extend(longsinowarp(age, 0.3, 0.5)))
                        .with(scale(), Vec3::splat(sinowarp(age, 0.1))),
                );
            }
        });
    query(age()).requires(is_firelick()).each_frame(|licks| {
        for (lick, t) in licks {
            entity::add_component(
                lick,
                scale(),
                Vec2::splat(2.5 * triwarp(t, 0.15)).extend(5. * sinowarp(t, 0.1)),
            );
            entity::mutate_component(lick, age(), |t| *t += 0.5 * *t * delta_time());
        }
    });
    spawn_query((just_looped(), is_firesmoke())).bind(|smokes| {
        for (smoke, _) in smokes {
            entity::add_component(smoke, rotation(), Quat::from_rotation_z(rr(0., PI * 2.)));
        }
    });

    // change_query(age())
    //     .track_change(age())
    //     .requires(is_firelick())
    //     .bind(|licks| {
    //         for (lick, age) in licks {
    //             entity::add_components(
    //                 lick,
    //                 Entity::new(), // no need!
    //             );
    //         }
    //     });

    if test {
        Entity::new()
            .with(translation(), Vec3::ZERO)
            .with(is_droplet_spawner(), ())
            .with(spawns_fire(), ())
            .spawn();
    }
}

fn spawn_test_camera() {
    use ambient_api::core::{
        camera::concepts::{
            PerspectiveInfiniteReverseCamera, PerspectiveInfiniteReverseCameraOptional,
        },
        transform::components::lookat_target,
    };

    PerspectiveInfiniteReverseCamera {
        optional: PerspectiveInfiniteReverseCameraOptional {
            aspect_ratio_from_window: Some(entity::resources()),
            main_scene: Some(()),
            translation: Some(Vec3::ONE * 7.),
            ..default()
        },
        ..PerspectiveInfiniteReverseCamera::suggested()
    }
    .make()
    .with(lookat_target(), vec3(0., 0., 1.))
    .spawn();
}
