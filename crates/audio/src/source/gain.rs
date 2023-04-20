use crate::Source;
use parking_lot::Mutex;
use std::fmt::Debug;
use std::sync::Arc;

pub trait GainValue: Clone {
    fn get_value(&self) -> f32;
}

impl GainValue for f32 {
    fn get_value(&self) -> f32 {
        *self
    }
}

impl GainValue for Arc<Mutex<f32>> {
    fn get_value(&self) -> f32 {
        *self.lock()
    }
}

#[derive(Debug, Clone)]
pub struct Gain<S, G>
where
    G: GainValue,
{
    source: S,
    gain: G,
}

impl<S: Source, G: GainValue> Gain<S, G> {
    pub fn new(source: S, gain: G) -> Self {
        Self { source, gain }
    }
}

impl<S, G> Source for Gain<S, G>
where
    S: Source,
    G: GainValue + Send,
{
    fn next_sample(&mut self) -> Option<crate::Frame> {
        Some(self.source.next_sample()? * self.gain.get_value())
    }

    fn sample_rate(&self) -> crate::SampleRate {
        self.source.sample_rate()
    }

    fn sample_count(&self) -> Option<u64> {
        self.source.sample_count()
    }
}
