use std::{collections::VecDeque, iter::repeat};

use glam::{vec2, Vec2};

use crate::{Frame, Source};

const BUFSIZE: usize = 1024;

#[derive(Debug, Clone)]
pub(crate) struct DynamicDelay<S> {
    source: S,
    pub delay: Vec2,
    // A small buffer of samples to be able to shift
    buffer: VecDeque<Frame>,
}

impl<S> DynamicDelay<S>
where
    S: Source,
{
    pub fn new(source: S) -> Self {
        Self {
            source,
            delay: Frame::ZERO,
            buffer: repeat(Frame::ZERO).take(BUFSIZE).collect(),
        }
    }
}

impl<S> Source for DynamicDelay<S>
where
    S: Source,
{
    #[inline(always)]
    fn next_sample(&mut self) -> Option<crate::Frame> {
        let last = BUFSIZE - 1;
        // Get a new frame from the source, and shift it into the buffer.
        let s = self.source.next_sample()?;
        self.buffer.pop_front();
        self.buffer.push_back(s);
        debug_assert_eq!(self.buffer.len(), BUFSIZE);

        let delay = self.delay;

        let delay_diff = delay.x - delay.y;

        let min_delay = delay.max_element().min(BUFSIZE as f32 - 1.0);

        let delay = vec2(
            min_delay - (-delay_diff).max(0.0),
            min_delay - (delay_diff).max(0.0),
        );

        let delay_fract = delay.fract();

        fn lerpf(a: f32, b: f32, t: f32) -> f32 {
            a + t * (b - a)
        }

        let l = lerpf(
            self.buffer[last - delay.x as usize].x,
            self.buffer[last - delay.x.ceil() as usize].x,
            delay_fract.x,
        );

        let r = lerpf(
            self.buffer[last - delay.y as usize].y,
            self.buffer[last - delay.y.ceil() as usize].y,
            delay_fract.y,
        );

        Some(vec2(l, r))
    }

    #[inline(always)]
    fn sample_rate(&self) -> crate::SampleRate {
        self.source.sample_rate()
    }

    #[inline(always)]
    fn sample_count(&self) -> Option<u64> {
        self.source.sample_count()
    }
}
