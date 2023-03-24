use ambient_api::{
    message::client::{MessageExt, Target},
    prelude::*,
};

#[main]
pub async fn main() -> EventResult {
    messages::Hello::new(false, "Hello, world from the client!").send(Target::NetworkUnreliable);
    messages::Hello::new(true, "Hello, world from the client!").send(Target::NetworkUnreliable);

    messages::Hello::subscribe(|source, data| {
        println!("{source:?}: {:?}", data);
        EventOk
    });

    let handled = State::new(false);
    messages::Local::subscribe({
        let handled = handled.clone();
        move |source, data| {
            println!("{source:?}: {data:?}");
            *handled.write() = true;

            EventOk
        }
    });
    run_async(async move {
        loop {
            if *handled.read() {
                break;
            }

            sleep(1.0).await;
            messages::Local::new("Hello!").send(Target::ModuleBroadcast);
        }
        EventOk
    });

    EventOk
}
