use std::sync::Arc;

use itertools::Itertools;
use parking_lot::Mutex;

use crate::{uniform, uniform_n, Frame, Source, Uniform};

/// Mixes two sources together
#[derive(Debug, Clone)]
pub struct Mix<L, R> {
    left: Uniform<L>,
    right: Uniform<R>,
}

impl<L, R> Mix<L, R>
where
    L: Source,
    R: Source,
{
    pub fn new(left: L, right: R) -> Self {
        let (left, right) = uniform(left, right);
        Self { left, right }
    }
}

impl<L, R> Source for Mix<L, R>
where
    L: Source,
    R: Source,
{
    fn next_sample(&mut self) -> Option<crate::Frame> {
        let l = self.left.next_sample()?;
        let r = self.right.next_sample()?;

        Some(l + r)
    }

    fn sample_rate(&self) -> crate::SampleRate {
        self.left.sample_rate()
    }

    fn sample_count(&self) -> Option<u64> {
        match (self.left.sample_count(), self.right.sample_count()) {
            (None, None) => None,
            (None, Some(v)) => Some(v),
            (Some(v), None) => Some(v),
            (Some(a), Some(b)) => Some(a.min(b)),
        }
    }
}

pub struct DynamicMix {
    sources: Box<[Uniform<Box<dyn Source>>]>,
    weights: Arc<Mutex<Box<[f32]>>>,
}

impl DynamicMix {
    pub fn new(sources: Vec<Box<dyn Source>>, weights: Arc<Mutex<Box<[f32]>>>) -> Self {
        Self {
            sources: uniform_n(sources).collect_vec().into_boxed_slice(),
            weights,
        }
    }
}

impl Source for DynamicMix {
    fn next_sample(&mut self) -> Option<crate::Frame> {
        let weights = self.weights.lock();
        let mut sample = Frame::ZERO;

        for (s, &w) in self.sources.iter_mut().zip(weights.iter()) {
            sample += s.next_sample()? * w;
        }

        Some(sample)
    }

    fn sample_rate(&self) -> crate::SampleRate {
        self.sources.first().map(|v| v.sample_rate()).unwrap_or(1)
    }

    fn sample_count(&self) -> Option<u64> {
        self.sources
            .iter()
            .map(|v| v.sample_count())
            .min()
            .unwrap_or_default()
    }
}
