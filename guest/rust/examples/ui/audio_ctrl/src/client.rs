use ambient_api::{components::core::layout::space_between_items, prelude::*};

#[element_component]
fn App(hooks: &mut Hooks, audio_player: audio::AudioPlayer) -> Element {
    let (f32_value, set_f32_value) = hooks.use_state(100.);

    let (panning, set_panning) = hooks.use_state(0.);

    FocusRoot::el([FlowColumn::el([
        Text::el("Amplitude:"),
        Slider {
            value: f32_value,
            on_change: Some(cb({
                let audio_player = audio_player.clone();
                move |v| {
                    set_f32_value(v);
                    &audio_player.set_amplitude(v / 100.);
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
            value: panning,
            on_change: Some(cb({
                let audio_player = audio_player.clone();
                move |v| {
                    set_panning(v);
                    &audio_player.set_panning(v);
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
                audio_player.play();
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
    let audio_player = audio::AudioPlayer::from_url(asset::url("assets/arpy01.wav").unwrap());
    App::el(audio_player).spawn_interactive();
}
