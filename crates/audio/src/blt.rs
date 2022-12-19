use std::{f32::consts::TAU, fmt::Debug};

use glam::Vec2;

use crate::{value::Value, SampleRate, Source};

#[derive(Debug, Clone)]
pub struct BltCoeffs {
    a1: f32,
    a2: f32,
    b0: f32,
    b1: f32,
    b2: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Hpf {
    pub freq: f32,
    // In dB/octave
    pub bandwidth: f32,
}

impl TransferFunction for Hpf {
    fn get_coeffs(&self, sample_freq: SampleRate) -> BltCoeffs {
        let w0 = TAU * self.freq / sample_freq as f32;
        let re = w0.cos();

        let alpha = w0.sin() * (2f32.ln() * 0.5 * self.bandwidth * w0 / w0.sin()).sinh();

        let b0 = (1.0 + re) / 2.0;
        let b1 = -1.0 - re;
        let b2 = b0;

        let a0 = 1.0 + alpha;
        let a1 = -2.0 * re;
        let a2 = 1.0 - alpha;

        // Normalization step
        BltCoeffs {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lpf {
    pub freq: f32,
    // In dB/octave
    pub bandwidth: f32,
}

impl TransferFunction for Lpf {
    fn get_coeffs(&self, sample_freq: SampleRate) -> BltCoeffs {
        let w0 = TAU * self.freq / sample_freq as f32;
        let re = w0.cos();

        let alpha = w0.sin() * (2f32.ln() * 0.5 * self.bandwidth * w0 / w0.sin()).sinh();
        // let alpha = w0.sin() * recip_q_from_bandwidth(self.rolloff, w0) * 0.5;

        let b0 = (1.0 - re) / 2.0;
        let b1 = 1.0 - re;
        let b2 = b0;

        let a0 = 1.0 + alpha;
        let a1 = -2.0 * re;
        let a2 = 1.0 - alpha;

        // Normalization step
        BltCoeffs {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bpf {
    pub freq: f32,
    pub bandwidth: f32,
}

impl TransferFunction for Bpf {
    fn get_coeffs(&self, sample_freq: SampleRate) -> BltCoeffs {
        let w0 = TAU * self.freq.max(0.001) / sample_freq as f32;
        let re = w0.cos();

        let alpha = w0.sin() * (2f32.ln() / 2.0 * self.bandwidth * w0 / w0.sin()).sinh();

        let b0 = alpha;
        let b1 = 0.0;
        let b2 = -alpha;

        let a0 = 1.0 + alpha;
        let a1 = -2.0 * re;
        let a2 = 1.0 - alpha;

        // Normalization step
        BltCoeffs {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }
}

pub trait TransferFunction {
    fn get_coeffs(&self, sample_freq: SampleRate) -> BltCoeffs;
}

#[derive(Debug, Clone)]
pub struct BilinearTransform<S, H, Vh>
where
    Vh: for<'x> Value<'x, Item = H>,
{
    source: S,
    c: BltCoeffs,
    filter: Vh,
    prev_filter: H,
    x1: Vec2,
    x2: Vec2,
    y1: Vec2,
    y2: Vec2,
}

impl<S, H, Vh> BilinearTransform<S, H, Vh>
where
    S: Source,
    H: Send + Clone + PartialEq + TransferFunction,
    Vh: for<'x> Value<'x, Item = H>,
{
    pub fn new(source: S, filter: Vh) -> Self {
        let sample_freq = source.sample_rate();
        let f = filter.get().clone();
        let c = f.get_coeffs(sample_freq);

        Self {
            source,
            c,
            x1: Vec2::ZERO,
            x2: Vec2::ZERO,
            y1: Vec2::ZERO,
            y2: Vec2::ZERO,
            prev_filter: f,
            filter,
        }
    }
}

impl<S, H, Vh> Source for BilinearTransform<S, H, Vh>
where
    S: Source,
    H: Send + Clone + PartialEq + TransferFunction,
    Vh: for<'x> Value<'x, Item = H>,
{
    fn next_sample(&mut self) -> Option<crate::Frame> {
        let filter = self.filter.get();
        if self.prev_filter != *filter {
            self.c = filter.get_coeffs(self.sample_rate());
            self.prev_filter = filter.clone();
        }
        drop(filter);

        let sample = self.source.next_sample()?;

        let y = self.c.b0 * sample + self.c.b1 * self.x1 + self.c.b2 * self.x2
            - self.c.a1 * self.y1
            - self.c.a2 * self.y2;

        // Slide
        self.x2 = self.x1;
        self.x1 = sample;

        self.y2 = self.y1;
        self.y1 = y;

        Some(y)
    }

    fn sample_rate(&self) -> SampleRate {
        self.source.sample_rate()
    }

    fn sample_count(&self) -> Option<u64> {
        self.source.sample_count()
    }
}
