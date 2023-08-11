use ambient_api::prelude::*;

use embers::afps_schema::{components, messages::*};

#[main]
pub fn main() {
    Shoot::subscribe(move |_, msg| {
        FireSound::new(msg.source).send_client_broadcast_unreliable();
    });

    FootOnGround::subscribe(move |_, msg| {
        if !entity::get_component(msg.source, components::player_jumping()).unwrap_or_default() {
            WalkSound::new(msg.source).send_client_broadcast_unreliable();
        }
    });

    Explosion::subscribe(move |_, msg| {
        println!("explosion msg got from server");
        Explosion::new(msg.pos).send_client_broadcast_unreliable();
    });
}
