use std::{f32::consts::TAU, time::Duration};

use ambient_audio::{AudioStream, Frame, SampleRate, SineWave, Source};
use ambient_sys::time::Instant;

#[derive(Debug, Clone)]
pub struct BinauralSine {
    freq: f32,
    cursor: usize,
}

impl Source for BinauralSine {
    fn next_sample(&mut self) -> Option<Frame> {
        let l = self.freq * self.cursor as f32 * TAU / 44100_f32;
        let r = self.freq * 0.5 * self.cursor as f32 * TAU / 44100_f32;
        self.cursor += 1;
        Some(Frame::new(l.sin(), r.sin()))
    }

    fn sample_rate(&self) -> SampleRate {
        44100
    }

    fn sample_count(&self) -> Option<u64> {
        None
    }
}

fn main() {
    let stream = AudioStream::new().unwrap();

    let mixer = stream.mixer();

    let bsine = BinauralSine {
        freq: 440.0,
        cursor: 0,
    }
    .take(Duration::from_secs(5));

    let source = SineWave::new(440.0)
        .take(Duration::from_millis(1500))
        .crossfade(bsine, Duration::from_millis(500));

    eprintln!("Duration: {:?}", source.duration());
    let start = Instant::now();
    let sound = mixer.play(source);

    sound.wait_blocking();
    eprintln!("Elapsed: {:?}", start.elapsed());
}
