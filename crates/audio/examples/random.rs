use std::time::Duration;

use elements_audio::{track::Track, AudioStream, Source};
use elements_std::time::IntoDuration;
use rand::{seq::SliceRandom, thread_rng};

fn main() -> color_eyre::Result<()> {
    let stream = AudioStream::new()?;

    let track1 = Track::from_wav(
        std::fs::read("example_assets/Thunder-Mike_Koenig-315681025.wav")
            .unwrap()
            .to_vec(),
    )?;

    let track2 = Track::from_vorbis(
        std::fs::read("example_assets/Apocalypse.ogg")
            .unwrap()
            .to_vec(),
    )?;

    let sources = vec![track1, track2];

    let mut rng = thread_rng();
    for _ in 0..5 {
        let a = sources
            .choose(&mut rng)
            .unwrap()
            .decode()
            .take(Duration::from_secs(5));

        let b = sources
            .choose(&mut rng)
            .unwrap()
            .decode()
            .take(Duration::from_secs(5));

        let c = sources
            .choose(&mut rng)
            .unwrap()
            .decode()
            .take(Duration::from_secs(5));

        let source = a.crossfade(b, 200.ms()).crossfade(c, 200.ms());

        eprintln!("Source is {:?} in duration", source.duration());

        stream.mixer().play(source).wait_blocking();
    }

    Ok(())
}
