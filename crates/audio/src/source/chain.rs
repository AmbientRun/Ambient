use super::uniform::{uniform, Uniform};
use crate::{Frame, SampleRate, Source};

#[derive(Debug, Clone)]
pub struct Chain<L, R> {
    left: Uniform<L>,
    right: Uniform<R>,
}

impl<L: Source, R: Source> Chain<L, R> {
    pub fn new(left: L, right: R) -> Self {
        // Convert left and right to the same sample rate
        let (left, right) = uniform(left, right);
        Self { left, right }
    }
}

impl<L: Source, R: Source> Source for Chain<L, R> {
    fn next_sample(&mut self) -> Option<Frame> {
        if let Some(s) = self.left.next_sample() {
            Some(s)
        } else {
            self.right.next_sample()
        }
    }

    fn sample_rate(&self) -> SampleRate {
        self.left.sample_rate()
    }

    fn sample_count(&self) -> Option<u64> {
        Some(self.left.sample_count()? + self.right.sample_count()?)
    }
}

#[cfg(test)]
mod test {

    use itertools::Itertools;

    use super::*;
    use crate::{BufferedSource, SampleIter};

    #[test]
    fn chain() {
        let l = BufferedSource::new([0.0, 1.0, 5.0, 2.0], 1, 4);
        let r = BufferedSource::new([0.0, 0.0, 2.0, 2.0, 3.0, 3.0], 2, 2);

        let source = Chain::new(l, r);
        // assert_eq!(source.channel_count(), 2);
        // assert_eq!(source.sample_rate(), 4);

        let samples = SampleIter::new(source).collect_vec();

        assert_eq!(
            samples,
            [
                Frame::new(0.0, 0.0),
                Frame::new(1.0, 1.0),
                Frame::new(5.0, 5.0),
                Frame::new(2.0, 2.0), //
                Frame::new(0.0, 0.0),
                Frame::new(1.0, 1.0),
                Frame::new(2.0, 2.0),
                Frame::new(2.5, 2.5),
                Frame::new(3.0, 3.0),
                Frame::new(3.0, 3.0),
            ]
        );
    }
}
