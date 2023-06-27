use ambient_api::prelude::*;

#[main]
pub fn main() {
    messages::Shoot::subscribe(move |_, msg| {
        messages::FireSound::new(msg.source).send_client_broadcast_unreliable();
    });
}
