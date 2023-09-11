use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use ambient_api::{
    core::{
        camera::concepts::{
            PerspectiveInfiniteReverseCamera, PerspectiveInfiniteReverseCameraOptional,
        },
        primitives::components::cube,
        rendering::components::color,
        transform::components::{lookat_target, scale, translation},
    },
    prelude::*,
};
use packages::this::messages::{Hello, Local};

#[main]
pub fn main() {
    // The main clientside WASM of the messaging example.
    //
    // It sets up a camera to look at the cubes that are spawned when messages are received (0).
    //
    // It sends messages to the server (1), and then prints out the response it gets back from the server,
    // spawning cubes to indicate that the message has been received (2).
    //
    // After that, it will register a listener for messages from other modules on this side (3), and then send a message to
    // other modules (e.g. `client_two.rs`) until it gets a response (4).

    // 0
    PerspectiveInfiniteReverseCamera {
        local_to_world: Mat4::IDENTITY,
        near: 0.1,
        projection: Mat4::IDENTITY,
        projection_view: Mat4::IDENTITY,
        active_camera: 0.0,
        inv_local_to_world: Mat4::IDENTITY,
        fovy: 1.0,
        aspect_ratio: 1.0,
        perspective_infinite_reverse: (),
        optional: PerspectiveInfiniteReverseCameraOptional {
            translation: Some(Vec3::ONE * 5.),
            main_scene: Some(()),
            aspect_ratio_from_window: Some(entity::resources()),
            ..default()
        },
    }
    .make()
    .with(lookat_target(), vec3(0., 0., 0.))
    .spawn();

    // 1
    Hello::new("Hello, world from the client!", false).send_server_unreliable();
    Hello::new("Hello, world from the client!", true).send_server_reliable();

    // 2
    Hello::subscribe(|ctx, data| {
        println!("{ctx:?}: {:?}", data);

        let source_reliable = data.source_reliable;
        Entity::new()
            .with(cube(), ())
            .with(
                translation(),
                vec3(0., if source_reliable { -1. } else { 1. }, 0.),
            )
            .with(scale(), Vec3::ONE)
            .with(
                color(),
                if source_reliable {
                    vec4(0., 1., 0., 1.)
                } else {
                    vec4(1., 1., 0., 1.)
                },
            )
            .spawn();
    });

    // 3
    let handled = Arc::new(AtomicBool::new(false));
    Local::subscribe({
        let handled = handled.clone();
        move |ctx, data| {
            handled.store(true, Ordering::SeqCst);
            println!("{ctx:?}: {data:?}");
        }
    });

    // 4
    run_async(async move {
        while !handled.load(Ordering::SeqCst) {
            sleep(1.0).await;
            Local::new("Hello!").send_local_broadcast(false)
        }
    });
}
