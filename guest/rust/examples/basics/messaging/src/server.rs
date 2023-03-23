use ambient_api::{
    message::server::{MessageExt, Source, Target},
    prelude::*,
};

#[main]
pub async fn main() -> EventResult {
    messages::Hello::subscribe(|source, data| {
        let Source::Network { user_id } = source else { return EventOk; };
        println!("{user_id}: {:?}", data);

        let source_reliable = data.source_reliable;

        messages::Hello {
            text: format!("{source_reliable}: Hello, world from the server!"),
            source_reliable: true,
        }
        .send(Target::NetworkTargetedReliable(user_id.clone()));

        messages::Hello {
            text: format!("{source_reliable}: Hello, world from the server!"),
            source_reliable: false,
        }
        .send(Target::NetworkTargetedUnreliable(user_id.clone()));

        messages::Hello {
            text: format!("{source_reliable}: Hello, world (everyone) from the server!"),
            source_reliable: true,
        }
        .send(Target::NetworkBroadcastReliable);

        EventOk
    });

    EventOk
}
