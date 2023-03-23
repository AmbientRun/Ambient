use ambient_api::{
    message::client::{MessageExt, Target},
    prelude::*,
};

#[main]
pub async fn main() -> EventResult {
    messages::Hello {
        text: "Hello, world from the client!".into(),
        source_reliable: false,
    }
    .send(Target::NetworkUnreliable);

    messages::Hello {
        text: "Hello, world from the client!".into(),
        source_reliable: true,
    }
    .send(Target::NetworkUnreliable);

    messages::Hello::subscribe(|source, data| {
        println!("{source:?}: {:?}", data);
        EventOk
    });

    EventOk
}
