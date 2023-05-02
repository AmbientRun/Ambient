use ambient_audio::{AudioStream, SineWave, Source};
use ambient_sys::time::Instant;
use std::time::Duration;
use tokio::{join, time::sleep};

#[tokio::main]
async fn main() {
    let stream = AudioStream::new().unwrap();

    let mixer = stream.mixer();

    let source = SineWave::new(440.0).take(Duration::from_secs(4));

    let source = SineWave::new(440.0)
        .gain(0.5)
        .take(Duration::from_millis(1500))
        .chain(source);

    let start = Instant::now();
    let sound = mixer.play(source);

    let source = SineWave::new(840.0)
        .gain(0.5)
        .take(Duration::from_millis(1500));

    let sound2 = mixer.play(source);

    // sleep(Duration::from_secs(5)).await;
    join!(sound.wait(), sound2.wait());

    eprintln!("Elapsed: {:?}", start.elapsed());
}
