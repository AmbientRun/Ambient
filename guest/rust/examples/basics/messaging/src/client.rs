use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use ambient_api::prelude::*;

#[main]
pub fn main() {
    messages::Hello::new(false, "Hello, world from the client!").send_server_unreliable();
    messages::Hello::new(true, "Hello, world from the client!").send_server_unreliable();

    messages::Hello::subscribe(|source, data| {
        println!("{source:?}: {:?}", data);
    });

    let handled = Arc::new(AtomicBool::new(false));
    messages::Local::subscribe({
        let handled = handled.clone();
        move |source, data| {
            handled.store(true, Ordering::SeqCst);
            println!("{source:?}: {data:?}");
        }
    });
    run_async(async move {
        while !handled.load(Ordering::SeqCst) {
            sleep(1.0).await;
            messages::Local::new("Hello!").send_local_broadcast()
        }
    });
}
