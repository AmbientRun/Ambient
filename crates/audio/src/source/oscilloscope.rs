use std::{mem, sync::Arc};

use glam::Vec2;
use parking_lot::Mutex;


use crate::{Frame, Source};

const DERIVATE_LEN: usize = 8;
#[derive(Default, Debug, Clone)]
pub struct Derivative {
    prev: [Vec2; DERIVATE_LEN],
}

impl Derivative {
    fn calculate_derivative(&self, new_value: Vec2) -> Vec2 {
        self.prev
            .iter()
            .map(|&v| new_value - v)
            .fold(Vec2::ZERO, |acc, x| acc + x)
            / DERIVATE_LEN as f32
    }

    // Pushes a new value, and returns the derivative
    fn push_value(&mut self, value: Vec2) -> Vec2 {
        let d = self.calculate_derivative(value);
        self.prev.copy_within(1..DERIVATE_LEN - 1, 0);
        self.prev[DERIVATE_LEN - 1] = value;
        d
    }
}

#[derive(Debug, Clone)]
pub struct Oscilloscope<S> {
    periods: usize,
    cur_periods: (usize, usize),
    source: S,

    prev_d: Vec2,
    derivative: Derivative,

    swapbuffer: Vec<Frame>,
    output: Arc<Mutex<Vec<Frame>>>,
}

impl<S> Oscilloscope<S> {
    pub fn new(source: S, periods: usize, output: Arc<Mutex<Vec<Frame>>>) -> Self {
        Self {
            periods,
            cur_periods: (0, 0),
            source,
            swapbuffer: Vec::new(),
            output,
            derivative: Default::default(),
            prev_d: Vec2::ZERO,
        }
    }
}

impl<S> Source for Oscilloscope<S>
where
    S: Source,
{
    fn next_sample(&mut self) -> Option<crate::Frame> {
        let sample = self.source.next_sample()?;

        let d = self.derivative.push_value(sample);

        if self.prev_d.x < 0.0 && d.x >= 0.0 {
            self.cur_periods.0 += 1;
        }

        if self.prev_d.y < 0.0 && d.y >= 0.0 {
            self.cur_periods.1 += 1;
        }

        self.prev_d = d;
        self.swapbuffer.push(sample);

        if self.cur_periods.0 >= self.periods && self.cur_periods.1 >= self.periods {
            self.cur_periods = (0, 0);
            mem::swap(&mut self.swapbuffer, &mut self.output.lock());
            self.swapbuffer.clear();
        }

        Some(sample)
    }

    fn sample_rate(&self) -> crate::SampleRate {
        self.source.sample_rate()
    }

    fn sample_count(&self) -> Option<u64> {
        self.source.sample_count()
    }
}
