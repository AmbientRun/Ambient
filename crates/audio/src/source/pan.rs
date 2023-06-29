use crate::Source;
use parking_lot::Mutex;
use std::f32::consts::FRAC_PI_2;
use std::fmt::Debug;
use std::sync::Arc;

fn compute_pan(pan: f32) -> (f32, f32) {
    let pan = (pan + 1.0) * 0.5 * FRAC_PI_2; // transform from [-1, 1] to [0, PI/2]
    (pan.cos(), pan.sin())
}

pub trait PanValue: Clone {
    fn get_value(&self) -> f32;
}

impl PanValue for f32 {
    fn get_value(&self) -> f32 {
        *self
    }
}

impl PanValue for Arc<Mutex<f32>> {
    fn get_value(&self) -> f32 {
        *self.lock()
    }
}

#[derive(Debug, Clone)]
pub struct Pan<S, P>
where
    P: PanValue,
{
    source: S,
    pan: P,
}

impl<S: Source, P: PanValue> Pan<S, P> {
    pub fn new(source: S, pan: P) -> Self {
        Self { source, pan }
    }
}

impl<S, P> Source for Pan<S, P>
where
    S: Source,
    P: PanValue + Send,
{
    fn next_sample(&mut self) -> Option<crate::Frame> {
        let (gain_left, gain_right) = compute_pan(self.pan.get_value());
        self.source.next_sample().map(|mut frame| {
            frame[0] *= gain_left;
            frame[1] *= gain_right;
            frame
        })
    }

    fn sample_rate(&self) -> crate::SampleRate {
        self.source.sample_rate()
    }

    fn sample_count(&self) -> Option<u64> {
        self.source.sample_count()
    }
}
