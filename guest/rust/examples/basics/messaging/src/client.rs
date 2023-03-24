use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use ambient_api::{
    message::client::{MessageExt, Target},
    prelude::*,
};

#[main]
pub async fn main() -> EventResult {
    messages::Hello::new(false, "Hello, world from the client!").send(Target::RemoteUnreliable);
    messages::Hello::new(true, "Hello, world from the client!").send(Target::RemoteUnreliable);

    messages::Hello::subscribe(|source, data| {
        println!("{source:?}: {:?}", data);
        EventOk
    });

    let handled = Arc::new(AtomicBool::new(false));
    messages::Local::subscribe({
        let handled = handled.clone();
        move |source, data| {
            handled.store(true, Ordering::SeqCst);
            println!("{source:?}: {data:?}");
            EventOk
        }
    });
    run_async(async move {
        while !handled.load(Ordering::SeqCst) {
            sleep(1.0).await;
            messages::Local::new("Hello!").send(Target::LocalBroadcast);
        }
        EventOk
    });

    EventOk
}
