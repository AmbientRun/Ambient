use std::time::Instant;

use elements_audio::{track::Track, AudioStream, Source};

fn main() {
    let track = Track::from_wav(
        std::fs::read("example_assets/thunder.wav")
            .unwrap()
            .to_vec(),
    )
    .unwrap();

    let stream = AudioStream::new().unwrap();

    let source = track.decode();
    eprintln!("Duration: {:?}", source.duration());
    let sound = stream.mixer().play(source);
    let now = Instant::now();
    sound.wait_blocking();
    eprintln!("Elapsed: {:?}", now.elapsed());
}
