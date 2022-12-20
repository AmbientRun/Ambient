use std::time::Duration;

use crate::{to_sample_index, Frame, SampleRate, Source};

#[derive(Debug, Clone)]
pub struct PadTo<S> {
    source: S,
    len: SampleRate,
    cursor: SampleRate,
}

impl<S> PadTo<S>
where
    S: Source,
{
    pub fn new(source: S, dur: Duration) -> Self {
        let len = to_sample_index(source.sample_rate(), dur);
        Self {
            source,
            len,
            cursor: 0,
        }
    }
}

impl<S> Source for PadTo<S>
where
    S: Source,
{
    #[inline]
    fn next_sample(&mut self) -> Option<Frame> {
        if let Some(s) = self.source.next_sample() {
            self.cursor += 1;
            Some(s)
        } else if self.cursor < self.len {
            self.cursor += 1;
            Some(Frame::ZERO)
        } else {
            None
        }
    }

    #[inline]
    fn sample_rate(&self) -> SampleRate {
        self.source.sample_rate()
    }

    #[inline]
    fn sample_count(&self) -> Option<u64> {
        Some(self.source.sample_count()?.max(self.len))
    }
}

#[cfg(test)]
mod test {
    use glam::vec2;
    use itertools::Itertools;

    use super::*;
    use crate::BufferedSource;

    #[test]
    fn pad_to() {
        let source = PadTo::new(
            BufferedSource::new([0.0, 0.0, 1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0], 2, 2),
            Duration::from_secs_f32(4.5),
        );

        let samples = source.samples_iter().collect_vec();

        pretty_assertions::assert_eq!(
            samples,
            [
                // 1
                vec2(0.0, 0.0),
                vec2(1.0, 1.0),
                // 2
                vec2(2.0, 2.0),
                vec2(3.0, 3.0),
                // 3 secs
                vec2(4.0, 4.0),
                vec2(0.0, 0.0),
                // 4 secs
                vec2(0.0, 0.0),
                vec2(0.0, 0.0),
                // 5 secs
                vec2(0.0, 0.0),
            ]
        )
    }
}
