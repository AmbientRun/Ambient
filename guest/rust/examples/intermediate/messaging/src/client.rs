use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use ambient_api::{
    core::{
        app::components::main_scene,
        camera::{
            components::aspect_ratio_from_window,
            concepts::make_perspective_infinite_reverse_camera,
        },
        primitives::components::cube,
        rendering::components::color,
        transform::{
            components::{lookat_target, scale, translation},
            concepts::make_transformable,
        },
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
    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with(main_scene(), ())
        .with(translation(), Vec3::ONE * 5.)
        .with(lookat_target(), vec3(0., 0., 0.))
        .spawn();

    // 1
    Hello::new("Hello, world from the client!", false).send_server_unreliable();
    Hello::new("Hello, world from the client!", true).send_server_reliable();

    // 2
    Hello::subscribe(|source, data| {
        println!("{source:?}: {:?}", data);

        let source_reliable = data.source_reliable;
        Entity::new()
            .with_merge(make_transformable())
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
        move |source, data| {
            handled.store(true, Ordering::SeqCst);
            println!("{source:?}: {data:?}");
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
