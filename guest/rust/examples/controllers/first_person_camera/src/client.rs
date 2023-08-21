use ambient_api::{
    core::{layout::components::space_between_items, messages::Frame},
    prelude::*,
};
use packages::this::{
    assets,
    components::{ball_ref, player_head_ref},
    messages::Input,
};

#[main]
fn main() {
    let mut cursor_lock = input::CursorLockGuard::new();
    let spatial_audio_player = audio::SpatialAudioPlayer::new();
    spatial_audio_player.set_looping(true);
    spatial_audio_player.set_amplitude(0.5);

    spawn_query((player_head_ref(), ball_ref())).bind(move |v| {
        for (_id, (head, ball)) in v {
            spatial_audio_player.set_listener(head);
            spatial_audio_player.play_sound_on_entity(assets::url("amen_break.ogg"), ball);
            run_async(async move {
                sleep(10.).await;
                println!("stop audio 10 seconds...");
                audio::stop(ball);
            });
        }
    });

    Frame::subscribe(move |_| {
        let input = input::get();
        if !cursor_lock.auto_unlock_on_escape(&input) {
            return;
        }

        let mut displace = Vec2::ZERO;
        if input.keys.contains(&KeyCode::W) {
            displace.y -= 1.0;
        }
        if input.keys.contains(&KeyCode::S) {
            displace.y += 1.0;
        }
        if input.keys.contains(&KeyCode::A) {
            displace.x -= 1.0;
        }
        if input.keys.contains(&KeyCode::D) {
            displace.x += 1.0;
        }

        Input::new(displace, input.mouse_delta).send_server_unreliable();
    });

    App.el().spawn_interactive();
}

#[element_component]
fn App(_hooks: &mut Hooks) -> Element {
    FlowColumn::el([
        Text::el("wsad to move; mouse to look around."),
        Text::el("the ball is a sound source, with HRTF spatial audio."),
        Text::el(
            "if the audio is jitter, add `-r` or `--release` to your `cargo` or `ambient` command.",
        ),
        Text::el("this is because HRTF is heavy."),
    ])
    .with_padding_even(STREET)
    .with(space_between_items(), 10.)
}
