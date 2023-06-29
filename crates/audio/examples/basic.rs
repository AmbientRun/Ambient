use ambient_audio::{AudioStream, SineWave, Source};
use ambient_sys::time::Instant;
use std::time::Duration;

fn main() {
    let stream = AudioStream::new().unwrap();

    let mixer = stream.mixer();

    let sr = mixer.inner.sample_rate;

    let source = SineWave::new(440.0).sr(sr).take(Duration::from_secs(4));

    let source = SineWave::new(440.0)
        .gain(0.5)
        .take(Duration::from_millis(1500))
        .chain(source);

    let start = Instant::now();
    let sound = mixer.play(source);

    sound.wait_blocking();
    eprintln!("Elapsed: {:?}", start.elapsed());
}
