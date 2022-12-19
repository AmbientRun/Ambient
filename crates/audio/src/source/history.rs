use std::{fmt::Debug, sync::Arc};

use circular_queue::CircularQueue;
use parking_lot::Mutex;

use crate::{Frame, Source};

pub struct History<S> {
    source: S,
    buf: Arc<Mutex<CircularQueue<Frame>>>,
    report_stride: usize,
    acc: usize,
    acc_samples: Frame,
}

impl<S> History<S>
where
    S: Source,
{
    pub fn new(source: S, freq: f32, buf: Arc<Mutex<CircularQueue<Frame>>>) -> Self {
        // Spacing in sample between each report
        let report_stride = (source.sample_rate() as f32 / freq).round() as usize;

        Self {
            source,
            buf,
            report_stride,
            acc: 0,
            acc_samples: Frame::ZERO,
        }
    }
}

impl<S> Debug for History<S>
where
    S: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("History")
            .field("source", &self.source)
            .finish()
    }
}

impl<S> Source for History<S>
where
    S: Source,
{
    fn next_sample(&mut self) -> Option<crate::Frame> {
        let sample = self.source.next_sample()?;

        self.acc_samples += sample.abs();

        if self.acc >= self.report_stride {
            self.acc = 0;

            // println!("Sampling: {sample}");
            self.buf
                .lock()
                .push(self.acc_samples / self.report_stride as f32);
            self.acc_samples = Frame::ZERO;
        }

        self.acc += 1;
        Some(sample)
    }

    fn sample_rate(&self) -> crate::SampleRate {
        self.source.sample_rate()
    }

    fn sample_count(&self) -> Option<u64> {
        self.source.sample_count()
    }
}
