use ambient_sys::time::Instant;

use ambient_audio::{track::Track, AudioStream, Source};
use ambient_native_std::IntoDuration;

fn main() -> color_eyre::Result<()> {
    let stream = AudioStream::new()?;
    let mixer = stream.mixer();

    let track = Track::from_wav(std::fs::read("example_assets/ak47.wav").unwrap().to_vec())?;

    eprintln!("Track: {track:?}");

    let slices = [
        0.8.secs()..1.5.secs(),
        5.5.secs()..7.secs(),
        9.5.secs()..10.secs(),
        13.8.secs()..14.642.secs(),
        18.secs()..19.5.secs(),
        23.3.secs()..28.secs(),
        29.9.secs()..33.secs(),
    ];

    for (i, slice) in slices.iter().enumerate().cycle() {
        let source = track.decode().slice(slice.clone());

        eprintln!("--------------\nPlaying slice {i} {:?}", source.duration());

        let sound = mixer.play(source);

        let now = Instant::now();
        sound.wait_blocking();
        eprintln!("Elapsed: {:?}", now.elapsed());
    }

    Ok(())
}
