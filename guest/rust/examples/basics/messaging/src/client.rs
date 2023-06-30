use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use ambient_api::prelude::*;

#[main]
pub fn main() {
    // The main clientside WASM of the messaging example.
    //
    // It sends messages to the server (1), and then prints out the response it gets back from the server (2).
    //
    // After that, it will register a listener for messages from other modules on this side (3), and then send a message to
    // other modules (e.g. `client_two.rs`) until it gets a response (4).

    // 1
    messages::Hello::new(false, "Hello, world from the client!").send_server_unreliable();
    messages::Hello::new(true, "Hello, world from the client!").send_server_reliable();

    // 2
    messages::Hello::subscribe(|source, data| {
        println!("{source:?}: {:?}", data);
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
            messages::Local::new("Hello!").send_local_broadcast(false)
        }
    });
}
