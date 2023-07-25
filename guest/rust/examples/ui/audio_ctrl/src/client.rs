use ambient_api::{
    components::core::{
        audio::{amplitude, panning, stop_now},
        layout::space_between_items,
    },
    prelude::*,
};

#[element_component]
fn App(hooks: &mut Hooks, audio_player: audio::AudioPlayer) -> Element {
    let (f32_value, set_f32_value) = hooks.use_state(100.);
    let (sound, set_sound) = hooks.use_state(None);
    let (pan, set_pan) = hooks.use_state(0.);
    hooks.use_frame({
        let set_sound = set_sound.clone();
        move |_world| {
            if let Some(s) = sound {
                if !entity::exists(s) {
                    set_sound(None);
                }
            }
        }
    });
    FocusRoot::el([FlowColumn::el([
        Text::el("Amplitude:"),
        Slider {
            value: f32_value,
            on_change: Some(cb({
                let audio_player = audio_player.clone();
                move |v| {
                    set_f32_value(v);
                    audio_player.set_amplitude(v / 100.);
                    if let Some(s) = sound {
                        if entity::exists(s) {
                            entity::add_component(s, amplitude(), v / 100.);
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
                move |v| {
                    set_pan(v);
                    audio_player.set_panning(v);
                    if let Some(s) = sound {
                        if entity::exists(s) {
                            entity::add_component(s, panning(), v);
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
            let set_sound = set_sound.clone();
            move |_| {
                let id = audio_player.play(asset::url("assets/amen_break.wav").unwrap());
                // mono ogg
                // let id = audio_player.play(
                //     asset::url("assets/455516__ispeakwaves__the-plan-upbeat-loop-no-voice-edit-mono-track.ogg"
                // ).unwrap());
                set_sound(Some(id));
            }
        })
        .disabled(sound.is_some())
        .toggled(true)
        .el(),
        Button::new("stop sound", {
            move |_| {
                if let Some(s) = sound {
                    if entity::exists(s) {
                        entity::add_component(s, stop_now(), ());
                        set_sound(None);
                    } else {
                        set_sound(None);
                    }
                }
            }
        })
        .disabled(sound.is_none())
        .toggled(true)
        .el(),
    ])])
    .with(space_between_items(), STREET)
    .with_padding_even(STREET)
}

#[main]
pub fn main() {
    let audio_player = audio::AudioPlayer::new();
    audio_player.set_looping(true); // try false
    App::el(audio_player).spawn_interactive();
}
