use ambient_api::{message::server as message, prelude::*};

#[main]
pub async fn main() -> EventResult {
    message::subscribe_bytes("hello", |source, data| {
        let message::Source::Network { user_id } = source else { return EventOk; };

        println!("{user_id}: {:?}", String::from_utf8(data));

        message::send(
            message::Target::NetworkTargetedReliable {
                user_id: user_id.clone(),
            },
            "hello",
            String::from("Hello, world from the server (unistream)!").into_bytes(),
        );

        message::send(
            message::Target::NetworkTargetedUnreliable {
                user_id: user_id.clone(),
            },
            "hello",
            String::from("Hello, world from the server (datagram)!").into_bytes(),
        );

        message::send(
            message::Target::NetworkBroadcastReliable,
            "broadcast",
            String::from("Hello, world (everyone) from the server (unistream)!").into_bytes(),
        );

        EventOk
    });

    EventOk
}
