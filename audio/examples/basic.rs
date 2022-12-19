use std::{
    f32::consts::TAU, time::{Duration, Instant}
};

use elements_audio::{AudioStream, Frame, SampleRate, SineWave, Source, TweenSineWave};

fn main() {
    let stream = AudioStream::new().unwrap();

    let mixer = stream.mixer();

    // let source = SineWave { freq: 440.0 }.take(Duration::from_secs(2));
    let bsine =
        TweenSineWave::new(tween::CubicOut::new(200.0..=440.0, 4.0)).take(Duration::from_secs(4));

    let source = SineWave::new(440.0)
        .gain(0.5)
        .take(Duration::from_millis(1500))
        .chain(bsine);

    let start = Instant::now();
    let sound = mixer.play(source);

    sound.wait_blocking();
    println!("Elapsed: {:?}", start.elapsed());
}
