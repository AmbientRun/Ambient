use ambient_api::{components::core::physics::linear_velocity, prelude::*};

#[main]
pub fn main() {
    let bonk = audio::load(asset::url("assets/bonk.ogg").unwrap());

    messages::Bonk::subscribe(move |_source, data| {
        let cube = data.cube;
        let vel = entity::get_component(cube, linear_velocity()).unwrap();
        let mut amp = (vel.x.abs() / 5.0).powf(2.0)
            + (vel.y.abs() / 5.0).powf(2.0)
            + (vel.z.abs() / 5.0).powf(2.0);
        amp = amp.sqrt().clamp(0.0, 1.0);
        amp = amp * amp;
        bonk.looping(false).scale(amp).play();
    });
}
