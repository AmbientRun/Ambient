use crate::{Param, Source};
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct OnePole<S, P>
where
    P: Param,
{
    source: S,
    freq: P,
    a: f32,
    b: f32,
    y1: crate::Frame,
}

impl<S: Source, P: Param> OnePole<S, P> {
    pub fn new(source: S, freq: P) -> Self {
        let rate = freq.get_value() / source.sample_rate() as f32;
        let b = (-2.0 * std::f32::consts::PI * rate).exp();
        let a = 1.0 - b;
        Self {
            source,
            freq,
            a,
            b,
            y1: crate::Frame::default(),
        }
    }
}

impl<S, P> Source for OnePole<S, P>
where
    S: Source,
    P: Param + Send,
{
    fn next_sample(&mut self) -> Option<crate::Frame> {
        let rate = self.freq.get_value() / self.source.sample_rate() as f32;
        self.b = (-2.0 * std::f32::consts::PI * rate).exp();
        self.a = 1.0 - self.b;
        let next = self.source.next_sample()?;
        let y = next * self.a + self.b * self.y1;
        self.y1 = y;
        Some(y)
    }

    fn sample_rate(&self) -> crate::SampleRate {
        self.source.sample_rate()
    }

    fn sample_count(&self) -> Option<u64> {
        self.source.sample_count()
    }
}
