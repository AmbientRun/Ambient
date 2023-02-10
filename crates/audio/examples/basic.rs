use std::time::{Duration, Instant};

use kiwi_audio::{AudioStream, SineWave, Source};

fn main() {
    let stream = AudioStream::new().unwrap();

    let mixer = stream.mixer();

    let source = SineWave::new(440.0).take(Duration::from_secs(4));

    let source = SineWave::new(440.0)
        .gain(0.5)
        .take(Duration::from_millis(1500))
        .chain(source);

    let start = Instant::now();
    let sound = mixer.play(source);

    sound.wait_blocking();
    eprintln!("Elapsed: {:?}", start.elapsed());
}
