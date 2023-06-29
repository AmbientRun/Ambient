use ambient_api::{
    components::core::{
        audio::{amplitude, panning},
        layout::space_between_items,
    },
    prelude::*,
};

#[element_component]
fn App(hooks: &mut Hooks, audio_player: audio::AudioPlayer) -> Element {
    let (f32_value, set_f32_value) = hooks.use_state(100.);
    let (sound, set_sound) = hooks.use_state(None);
    let (pan, set_pan) = hooks.use_state(0.);
    FocusRoot::el([FlowColumn::el([
        Text::el("Amplitude:"),
        Slider {
            value: f32_value,
            on_change: Some(cb({
                let audio_player = audio_player.clone();
                let sound = sound.clone();
                move |v| {
                    set_f32_value(v);
                    audio_player.set_amplitude(v / 100.);
                    if sound.is_some() {
                        if entity::exists(sound.unwrap()) {
                            entity::add_component(sound.unwrap(), amplitude(), v / 100.);
                        }
                    }
                }
            })),
            min: 0.0,
            max: 100.,
            width: 100.,
            logarithmic: false,
            round: Some(2),
            suffix: Some("%"),
        }
        .el(),
        Text::el("Panning:"),
        Slider {
            value: pan,
            on_change: Some(cb({
                let audio_player = audio_player.clone();
                let sound = sound.clone();
                move |v| {
                    set_pan(v);
                    audio_player.set_panning(v);
                    if sound.is_some() {
                        if entity::exists(sound.unwrap()) {
                            entity::add_component(sound.unwrap(), panning(), v);
                        }
                    }
                }
            })),
            min: -1.0,
            max: 1.0,
            width: 100.,
            logarithmic: false,
            round: Some(4),
            suffix: None,
        }
        .el(),
        Button::new("play sound", {
            move |_| {
                let id =
                    audio_player.play(asset::url("assets/316829__lalks__ferambie.ogg").unwrap());
                set_sound(Some(id));
            }
        })
        .toggled(true)
        .style(ButtonStyle::Primary)
        .el(),
    ])])
    .with(space_between_items(), STREET)
    .with_padding_even(STREET)
}

#[main]
pub fn main() {
    let audio_player = audio::AudioPlayer::new();
    App::el(audio_player).spawn_interactive();
}
