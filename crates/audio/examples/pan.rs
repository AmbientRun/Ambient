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
        std::fs::read("../../guest/rust/examples/basics/physics/assets/bonk.ogg")
            .unwrap()
            .to_vec(),
    )
    .unwrap();

    let source = source
        .decode()
        .pan(0.0)
        .repeat()
        .take(Duration::from_secs(5))
        .chain(
            source
                .decode()
                .pan(-0.999)
                .repeat()
                .take(Duration::from_secs(5)),
        )
        .chain(
            source
                .decode()
                .pan(1.0)
                .repeat()
                .take(Duration::from_secs(5)),
        )
        .repeat();

    let sound = mixer.play(source);
    sound.wait_blocking();
}
