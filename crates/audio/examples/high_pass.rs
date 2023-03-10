use std::time::Duration;

use ambient_audio::{track::Track, AudioStream, Source};

fn main() {
    let stream = AudioStream::new().unwrap();

    let mixer = stream.mixer();

    // let ambience = Track::from_wav(
    //     Cursor::new(include_bytes!("../assets/ambience.wav")),
    //     "Ambience".into(),
    // )
    // .unwrap()
    // .as_source();

    let source = Track::from_vorbis(
        std::fs::read("example_assets/footstep04.ogg")
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    // .gain(0.25)
    // .mix(SineWave::new(659.25).gain(0.25))
    // .mix(SineWave::new(783.99).gain(0.25));

    let source = source
        .decode()
        .repeat()
        .take(Duration::from_secs(5))
        .chain(
            source
                .decode()
                .high_pass(2800.0, 3.0)
                .repeat()
                .take(Duration::from_secs(5)),
        )
        .repeat();

    let sound = mixer.play(source);
    sound.wait_blocking();
}
