use ambient_api::{message::client as message, prelude::*};

#[main]
pub async fn main() -> EventResult {
    message::send(
        message::Target::NetworkUnreliable,
        "hello",
        "Hello, world from the client (datagram)!".as_bytes(),
    );

    message::send(
        message::Target::NetworkReliable,
        "hello",
        "Hello, world from the client (unistream)!".as_bytes(),
    );

    message::subscribe_bytes("hello", |source, data| {
        println!("{source:?}: {:?}", String::from_utf8(data));
        EventOk
    });

    message::subscribe_bytes("broadcast", |source, data| {
        println!("{source:?}: {:?}", String::from_utf8(data));
        EventOk
    });

    EventOk
}
