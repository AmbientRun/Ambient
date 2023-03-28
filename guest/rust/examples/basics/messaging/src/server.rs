use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use ambient_api::{
    message::server::{MessageExt, Source, Target},
    prelude::*,
};

#[main]
pub fn main() -> ResultEmpty {
    messages::Hello::subscribe(|source, data| {
        let Source::Remote { user_id } = source else { return OkEmpty; };
        println!("{user_id}: {:?}", data);

        let source_reliable = data.source_reliable;

        messages::Hello::new(
            true,
            format!("{source_reliable}: Hello, world from the server!"),
        )
        .send(Target::RemoteTargetedReliable(user_id.clone()));

        messages::Hello::new(
            false,
            format!("{source_reliable}: Hello, world from the server!"),
        )
        .send(Target::RemoteTargetedUnreliable(user_id.clone()));

        messages::Hello::new(
            true,
            format!("{source_reliable}: Hello, world (everyone) from the server!"),
        )
        .send(Target::RemoteBroadcastReliable);

        OkEmpty
    });

    let handled = Arc::new(AtomicBool::new(false));
    messages::Local::subscribe({
        let handled = handled.clone();
        move |source, data| {
            handled.store(true, Ordering::SeqCst);
            println!("{source:?}: {data:?}");
            OkEmpty
        }
    });
    run_async(async move {
        while !handled.load(Ordering::SeqCst) {
            sleep(1.0).await;
            messages::Local::new("Hello!").send(Target::LocalBroadcast);
        }
        OkEmpty
    });

    OkEmpty
}
