use std::time::Duration;

use super::uniform::{uniform, Uniform};
use crate::{Frame, SampleRate, Source};

#[derive(Debug, Clone)]
pub struct Crossfade<L, R> {
    left: Uniform<L>,
    right: Uniform<R>,
    start: u64,
    end: u64,
    pos: u64,
}

impl<L: Source, R: Source> Crossfade<L, R> {
    pub fn new(left: L, right: R, dur: Duration) -> Self {
        // Convert left and right to the same sample rate
        let (left, right) = uniform(left, right);

        // in number of frames
        let dur = (left.sample_rate() * dur.as_nanos() as u64) / 1_000_000_000;
        let start = left.sample_count().unwrap_or(u64::MAX) - dur;
        let end = left.sample_count().unwrap_or(u64::MAX);

        Self {
            left,
            right,
            start,
            end,
            pos: 0,
        }
    }
}

impl<L: Source, R: Source> Source for Crossfade<L, R> {
    fn next_sample(&mut self) -> Option<Frame> {
        let pos = self.pos;
        if pos >= self.end {
            self.right.next_sample()
        } else if pos >= self.start {
            let l = self.left.next_sample().unwrap();
            let r = self.right.next_sample()?;

            let t = (pos - self.start) as f32 / (self.end - self.start) as f32;

            self.pos += 1;
            Some(l + t * (r - l))
        } else {
            self.pos += 1;
            Some(self.left.next_sample().unwrap())
        }
    }

    fn sample_rate(&self) -> SampleRate {
        self.left.sample_rate()
    }

    fn sample_count(&self) -> Option<u64> {
        Some(self.left.sample_count()? - (self.end - self.start) + self.right.sample_count()?)
    }
}

#[cfg(test)]
mod test {

    use itertools::Itertools;

    use super::*;
    use crate::{BufferedSource, SampleIter};

    #[test]
    fn crossfade() {
        let l = BufferedSource::new([0.0, 1.0, 5.0, 2.0], 1, 4);
        let r = BufferedSource::new([0.0, 0.0, 2.0, 2.0, 3.0, 3.0], 2, 2);

        let source = Crossfade::new(l, r, Duration::from_millis(500));
        // assert_eq!(source.channel_count(), 2);
        // assert_eq!(source.sample_rate(), 4);

        let samples = SampleIter::new(source).collect_vec();

        assert_eq!(
            samples,
            [
                Frame::new(0.0, 0.0),
                Frame::new(1.0, 1.0), //
                Frame::new(5.0, 5.0),
                Frame::new(1.5, 1.5),
                Frame::new(2.0, 2.0),
                Frame::new(2.5, 2.5),
                Frame::new(3.0, 3.0),
                Frame::new(3.0, 3.0)
            ]
        );
    }
}
