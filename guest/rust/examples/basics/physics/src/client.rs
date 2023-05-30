use ambient_api::prelude::*;

#[main]
pub fn main() {
    spawn_query((components::camera_ref())).bind(|cameras|{
        for (camera, ()) in cameras {
            println!("camera as listener: {:?}", camera);
            spatial_audio::set_listener(camera);
        }
    });
    messages::Bonk::subscribe(move |_source, data| {
        spatial_audio::set_emitter(data.emitter);
        spatial_audio::play_sound_on_entity(asset::url("assets/bonk.ogg").unwrap(), data.emitter);
    });
}
