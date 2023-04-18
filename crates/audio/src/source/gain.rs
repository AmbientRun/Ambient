use crate::Source;
use parking_lot::Mutex;
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Gain<S> {
    source: S,
    gain: Arc<Mutex<f32>>,
}

impl<S: Source> Gain<S> {
    pub fn new(source: S, gain: Arc<Mutex<f32>>) -> Self {
        Self { gain, source }
    }
}

impl<S> Source for Gain<S>
where
    S: Source,
{
    fn next_sample(&mut self) -> Option<crate::Frame> {
        Some(self.source.next_sample()? * *self.gain.lock())
    }

    fn sample_rate(&self) -> crate::SampleRate {
        self.source.sample_rate()
    }

    fn sample_count(&self) -> Option<u64> {
        self.source.sample_count()
    }
}
