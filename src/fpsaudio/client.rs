use ambient_api::prelude::*;

#[main]
pub fn main() {
    messages::FireSound::subscribe(move |_, msg| {
        let fire_sound_url = asset::url("assets/sound/m4a1.ogg").unwrap();
        let mut firesound = audio::load(fire_sound_url.clone());
        let whoshoot = msg.source;
        let listerner = player::get_local();

        // hrtf doesn't work well here
        // spatial_audio::set_emitter(msg.source);
        // spatial_audio::set_listener(player::get_local());
        // spatial_audio::play_sound_on_entity(fire_sound_url, msg.source);

        let pos_shoot = entity::get_component(whoshoot, translation()).unwrap();
        let pos_listen = entity::get_component(listerner, translation()).unwrap();
        let distance = (pos_listen - pos_shoot).length();
        // println!("distance.log2(): {}", distance.log2());
        // run_async(async move {
        //     let s = distance.clone() / 1000.0;

        //     sleep(s).await;
        firesound
            .volume(
                ({
                    if distance <= 1.0 {
                        1.0
                    } else {
                        1.0 / distance.log2()
                    }
                })
                .clamp(0.0, 1.0),
            )
            .play();
    });
    // });
}
