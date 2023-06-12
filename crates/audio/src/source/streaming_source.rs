use glam::Vec2;

use crate::{Frame, SampleRate, Source};

/// A source backed by a buffer of samples
#[derive(Clone)]
pub struct StreamingSource<I> {
    iter: I,
    sample_rate: SampleRate,
}

impl<I> std::fmt::Debug for StreamingSource<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamingSource")
            .field("sample_rate", &self.sample_rate)
            .finish()
    }
}

impl<I> StreamingSource<I> {
    pub fn new<T: IntoIterator<IntoIter = I>>(iter: T, sample_rate: SampleRate) -> Self {
        Self {
            iter: iter.into_iter(),
            sample_rate,
        }
    }
}

impl<I> Source for StreamingSource<I>
where
    I: Send + Iterator<Item = Vec2> + ExactSizeIterator,
{
    #[inline]
    fn next_sample(&mut self) -> Option<Frame> {
        self.iter.next()
    }

    #[inline]
    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    #[inline]
    fn sample_count(&self) -> Option<u64> {
        // eprintln!("Iter: {}", self.iter.len());
        Some(self.iter.len().try_into().unwrap())
    }
}
