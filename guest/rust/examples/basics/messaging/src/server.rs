use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use ambient_api::{
    components::core::{
        primitives::cube,
        rendering::color,
        transform::{scale, translation},
    },
    concepts::make_transformable,
    prelude::*,
};

#[main]
pub fn main() {
    // The main serverside WASM of the messaging example.
    //
    // It listens to messages from the client (1), and then sends targeted messages to that client, and a broadcast message
    // to every client (2).
    //
    // After that, it will register a listener for messages from other modules on this side (3), and then send a message to
    // other modules (e.g. `server_two.rs`) until it gets a response (4).
    //
    // A cube is spawned to indicate that the message has been received from the client.
    // See <https://github.com/AmbientRun/Ambient/issues/590> for more details.

    // 1
    messages::Hello::subscribe(|source, data| {
        let Some(user_id) = source.client_user_id() else { return; };
        println!("{user_id}: {:?}", data);

        let source_reliable = data.source_reliable;

        // 2
        messages::Hello::new(
            true,
            format!("{source_reliable}: Hello, world from the server!"),
        )
        .send_client_targeted_reliable(user_id.clone());

        messages::Hello::new(
            false,
            format!("{source_reliable}: Hello, world from the server!"),
        )
        .send_client_targeted_unreliable(user_id);

        messages::Hello::new(
            true,
            format!("{source_reliable}: Hello, world (everyone) from the server!"),
        )
        .send_client_broadcast_reliable();

        Entity::new()
            .with_merge(make_transformable())
            .with_default(cube())
            .with(
                translation(),
                vec3(if source_reliable { -1. } else { 1. }, 0., 0.),
            )
            .with(scale(), Vec3::ONE)
            .with(
                color(),
                if source_reliable {
                    vec4(1., 0., 0., 1.)
                } else {
                    vec4(0., 0., 1., 1.)
                },
            )
            .spawn();
    });

    // 3
    let handled = Arc::new(AtomicBool::new(false));
    messages::Local::subscribe({
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
            messages::Local::new("Hello!").send_local_broadcast(true);
        }
    });
}
