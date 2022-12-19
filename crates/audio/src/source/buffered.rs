use std::{marker::PhantomData};

use cpal::Sample;

use crate::{ChannelCount, Frame, SampleRate, Source};

/// A source backed by a buffer of samples
#[derive(Clone)]
pub struct BufferedSource<A, S> {
    /// Interlaced channels
    buf: A,
    cursor: usize,
    channel_count: ChannelCount,
    sample_rate: SampleRate,
    _marker: PhantomData<S>,
}

impl<A, S> std::fmt::Debug for BufferedSource<A, S>
where
    A: AsRef<[S]>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BufferedSource")
            .field("buf", &self.buf.as_ref().len())
            .field("cursor", &self.cursor)
            .field("channel_count", &self.channel_count)
            .field("sample_rate", &self.sample_rate)
            .finish()
    }
}

impl<A, S> BufferedSource<A, S>
where
    A: AsRef<[S]>,
{
    pub fn new(buf: A, channel_count: ChannelCount, sample_rate: SampleRate) -> Self {
        Self {
            buf,
            cursor: 0,
            channel_count,
            sample_rate,
            _marker: PhantomData,
        }
    }
}

impl<A, S> Source for BufferedSource<A, S>
where
    S: Sample + Send + Sync + 'static,
    A: Send + AsRef<[S]>,
{
    #[inline]
    fn next_sample(&mut self) -> Option<Frame> {
        let buf = self.buf.as_ref();
        if self.channel_count == 1 {
            let v = buf.get(self.cursor)?.to_f32();
            self.cursor += 1;
            Some(Frame::splat(v))
        } else {
            let l = buf.get(self.cursor)?.to_f32();
            let r = buf.get(self.cursor + 1).unwrap().to_f32();
            self.cursor += 2;
            Some(Frame::new(l, r))
        }
    }

    #[inline]
    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    #[inline]
    fn sample_count(&self) -> Option<u64> {
        Some(self.buf.as_ref().len() as u64 / self.channel_count as u64)
    }
}
