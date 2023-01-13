use std::{mem, ops::RangeBounds, time::Duration};

use crate::{to_sample_index, Source};

#[derive(Debug, Clone)]
pub struct Slice<S> {
    source: S,
    to_skip: u64,
    n: u64,
}

impl<S> Slice<S>
where
    S: Source,
{
    pub fn new(source: S, range: impl RangeBounds<Duration>) -> Self {
        let sample_rate = source.sample_rate();
        let start = match range.start_bound() {
            std::ops::Bound::Included(start) => to_sample_index(sample_rate, *start),
            std::ops::Bound::Excluded(start) => to_sample_index(sample_rate, *start) + 1,
            std::ops::Bound::Unbounded => u64::MAX,
        };

        let end = match range.end_bound() {
            std::ops::Bound::Included(end) => to_sample_index(sample_rate, *end) + 1,
            std::ops::Bound::Excluded(end) => to_sample_index(sample_rate, *end),
            std::ops::Bound::Unbounded => u64::MAX,
        };

        let to_skip = start;
        let n = end - start;

        Self { source, to_skip, n }
    }
}
impl<S> Source for Slice<S>
where
    S: Source,
{
    #[inline]
    fn next_sample(&mut self) -> Option<crate::Frame> {
        if self.to_skip > 0 {
            for _ in 0..mem::take(&mut self.to_skip) {
                self.source.next_sample()?;
            }
        }

        if self.n > 0 {
            self.n -= 1;
            self.source.next_sample()
        } else {
            None
        }
    }

    #[inline]
    fn sample_rate(&self) -> crate::SampleRate {
        self.source.sample_rate()
    }

    #[inline]
    fn sample_count(&self) -> Option<u64> {
        let sample_count = self.source.sample_count();
        match sample_count {
            Some(sample_count) => Some((sample_count - self.to_skip).min(self.n)),
            None if self.n != u64::MAX => Some(self.n),
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {

    use itertools::Itertools;

    use super::*;
    use crate::{BufferedSource, Frame};

    #[test]
    fn slice_full() {
        let source = BufferedSource::new(
            [0.0, 0.0, 1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0, 5.0, 5.0],
            2,
            2,
        )
        .slice(Duration::from_millis(500)..Duration::from_millis(2100));

        let samples = source.samples_iter().collect_vec();
        assert_eq!(
            samples,
            [
                Frame::new(1.0, 1.0),
                Frame::new(2.0, 2.0),
                Frame::new(3.0, 3.0)
            ]
        );
    }

    #[test]
    fn slice_full_incl() {
        let source = BufferedSource::new(
            [0.0, 0.0, 1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0, 5.0, 5.0],
            2,
            2,
        )
        .slice(Duration::from_millis(500)..=Duration::from_millis(2100));

        let samples = source.samples_iter().collect_vec();
        assert_eq!(
            samples,
            [
                Frame::new(1.0, 1.0),
                Frame::new(2.0, 2.0),
                Frame::new(3.0, 3.0),
                Frame::new(4.0, 4.0),
            ]
        );
    }
    #[test]
    fn slice_partial() {
        let source = BufferedSource::new(
            [0.0, 0.0, 1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0, 5.0, 5.0],
            2,
            2,
        )
        .slice(Duration::from_millis(500)..Duration::from_millis(5000));
        assert_eq!(source.sample_count(), Some(5));

        let samples = source.samples_iter().collect_vec();
        assert_eq!(
            samples,
            [
                Frame::new(1.0, 1.0),
                Frame::new(2.0, 2.0),
                Frame::new(3.0, 3.0),
                Frame::new(4.0, 4.0),
                Frame::new(5.0, 5.0),
            ]
        );
    }
}
