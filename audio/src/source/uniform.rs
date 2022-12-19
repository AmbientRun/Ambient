use crate::{ChannelCount, Frame, SampleConversion, SampleRate, Source};

/// Adapter which transform a source to another channel count and sample rate
#[derive(Debug, Clone)]
pub struct Uniform<S> {
    inner: Result<SampleConversion<S>, S>,
}

impl<S: Source> Uniform<S> {
    pub fn new(source: S, dst_rate: SampleRate) -> Self {
        let src_rate = source.sample_rate();

        if src_rate == dst_rate {
            Self { inner: Err(source) }
        } else {
            Self {
                inner: Ok(SampleConversion::new(source, dst_rate)),
            }
        }
    }
}

/// Convert two different sources to the same sample rate and channel count
pub fn uniform<L: Source, R: Source>(left: L, right: R) -> (Uniform<L>, Uniform<R>) {
    let max_rate = left.sample_rate().max(right.sample_rate());

    (Uniform::new(left, max_rate), Uniform::new(right, max_rate))
}

pub fn uniform_n(sources: Vec<Box<dyn Source>>) -> impl Iterator<Item = Uniform<Box<dyn Source>>> {
    let max_rate = sources.iter().map(|v| v.sample_rate()).max().unwrap_or(1);

    sources.into_iter().map(move |v| Uniform::new(v, max_rate))
}

impl<S: Source> Source for Uniform<S> {
    #[inline(always)]
    fn next_sample(&mut self) -> Option<Frame> {
        match &mut self.inner {
            Ok(v) => v.next_sample(),
            Err(v) => v.next_sample(),
        }
    }

    fn sample_buffered(&mut self, output: &mut [Frame]) -> usize {
        match &mut self.inner {
            Ok(v) => v.sample_buffered(output),
            Err(v) => v.sample_buffered(output),
        }
    }

    #[inline(always)]
    fn sample_rate(&self) -> SampleRate {
        match &self.inner {
            Ok(v) => v.dst_rate(),
            Err(v) => v.sample_rate(),
        }
    }

    #[inline(always)]
    fn sample_count(&self) -> Option<u64> {
        match &self.inner {
            Ok(v) => v.sample_count(),
            Err(v) => v.sample_count(),
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use itertools::Itertools;

    use super::*;
    use crate::{BufferedSource, SampleIter};

    #[test]
    fn mono_to_stereo2() {
        let l = BufferedSource::new([0.0, 1.0, 5.0, 2.0], 1, 4);
        let source = Uniform::new(l, 4);
        let samples = SampleIter::new(source).collect_vec();

        assert_eq!(
            samples,
            [
                Frame::new(0.0, 0.0),
                Frame::new(1.0, 1.0),
                Frame::new(5.0, 5.0),
                Frame::new(2.0, 2.0)
            ]
        )
    }
}
