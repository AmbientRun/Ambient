use std::time::Duration;

use kiwi_audio::{blt::Bpf, track::Track, value::Constant, AudioStream, Source};

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

    let source = source
        .decode()
        .low_pass(1000.0, 6.0)
        .repeat()
        .take(Duration::from_secs(5))
        .chain(
            source
                .decode()
                .high_pass(1000.0, 6.0)
                .repeat()
                .take(Duration::from_secs(5)),
        )
        .chain(
            source
                .decode()
                .blt(Constant(Bpf {
                    freq: 1000.0,
                    bandwidth: 3.0,
                }))
                .repeat()
                .take(Duration::from_secs(5)),
        )
        .repeat();

    let sound = mixer.play(source);
    sound.wait_blocking();
}
