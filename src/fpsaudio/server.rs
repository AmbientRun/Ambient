use ambient_api::prelude::*;

use afps::{
    afps_fpsaudio::messages::{Explosion, FireSound, FootOnGround, WalkSound},
    afps_fpsrule::messages::Shoot,
};

#[main]
pub fn main() {
    Shoot::subscribe(move |_, msg| {
        FireSound::new(msg.source).send_client_broadcast_unreliable();
    });

    FootOnGround::subscribe(move |_, msg| {
        WalkSound::new(msg.source).send_client_broadcast_unreliable();
    });

    Explosion::subscribe(move |_, msg| {
        println!("explosion msg got from server");
        Explosion::new(msg.pos).send_client_broadcast_unreliable();
    });
}
