use ambient_api::prelude::*;

#[main]
pub fn main() {

    audio::add_sound("bonk", asset::url("assets/bonk.ogg").unwrap());

    messages::Bonk::subscribe(|source, data| {
        println!("[{source:?}] sent a msg => {:?}", data);
        let mut amp = (data.vel.x.abs() / 5.0).powf(2.0)
        + (data.vel.y.abs() / 5.0).powf(2.0)
        + (data.vel.z.abs() / 5.0).powf(2.0);
        amp = amp.sqrt().clamp(0.0, 1.0);
        amp = amp * amp;
        audio::get("bonk").looping(false).scale(amp).play();
    });
}