use std::fmt::Debug;

use glam::Vec2;

use crate::{Frame, Source};

/// Buffered reader of a source.
///
/// This will only call `S::sample_buffered`
#[derive(Clone)]
pub struct BufferedReader<S> {
    buf: Vec<Frame>,
    source: S,
    cur: usize,
}

impl<S> std::fmt::Debug for BufferedReader<S>
where
    S: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BufferedReader")
            .field("buf", &self.buf.len())
            .field("source", &self.source)
            .field("cur", &self.cur)
            .finish()
    }
}

impl<S> BufferedReader<S>
where
    S: Source,
{
    pub fn new(source: S) -> Self {
        Self {
            buf: Vec::new(),
            source,
            cur: 0,
        }
    }

    fn fill(&mut self) -> usize {
        self.buf.fill(Vec2::ZERO);
        let read = self.source.sample_buffered(&mut self.buf);
        if read < self.buf.len() {
            self.buf.truncate(read);
        }

        self.cur = 0;
        read
    }

    // The number of samples which remain to read
    fn remaining(&self) -> usize {
        self.buf.len() - self.cur
    }
}

impl<S> Source for BufferedReader<S>
where
    S: Source,
{
    #[inline]
    fn next_sample(&mut self) -> Option<Frame> {
        if self.cur < self.buf.len() {
            let s = self.buf[self.cur];
            self.cur += 1;
            Some(s)
        } else if self.fill() > 0 {
            self.cur += 1;
            Some(self.buf[0])
        } else {
            None
        }
    }

    fn sample_buffered(&mut self, output: &mut [Frame]) -> usize {
        let mut rem = self.remaining();

        if rem == 0 {
            self.buf.resize(output.len(), Default::default());

            let filled = self.fill();

            for (a, b) in output.iter_mut().zip(&self.buf) {
                *a += *b;
            }

            self.cur = filled;

            filled
        } else {
            let mut read = 0;
            let taken = output.len().min(rem);
            for sample in output.iter_mut().take(rem) {
                *sample += self.buf[self.cur];
                self.cur += 1;
            }

            read += taken;
            rem -= taken;

            // If output > rem we need to fill the buffer once more

            self.buf.resize(rem, Default::default());

            let filled = self.fill();
            let count = (output.len() - taken).min(filled);
            for sample in &mut output[taken..taken + count] {
                *sample += self.buf[self.cur];
                self.cur += 1;
            }

            read += count;
            read
        }
    }

    #[inline]
    fn sample_rate(&self) -> crate::SampleRate {
        self.source.sample_rate()
    }

    #[inline]
    fn sample_count(&self) -> Option<u64> {
        self.source.sample_count()
    }
}

#[cfg(test)]
mod test {
    use std::iter::repeat;

    use glam::Vec2;
    use itertools::Itertools;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::BufferedSource;

    #[test]
    fn buffered_reader() {
        color_eyre::install().ok();
        let mut source = BufferedReader::new(BufferedSource::new(
            (0..500).map(|v| v as f32).collect_vec(),
            1,
            1,
        ));
        let mut output = vec![Vec2::ZERO; 1024];
        let read = source.sample_buffered(&mut output);
        assert_eq!(read, 500);

        let expected = (0..500)
            .map(|v| Vec2::splat(v as f32))
            .chain(repeat(Vec2::ZERO))
            .take(1024)
            .collect_vec();

        assert_eq!(output, expected);
    }

    #[test]
    fn partial_buffered() {
        let mut source = BufferedReader::new(BufferedSource::new(
            (0..500).map(|v| v as f32).collect_vec(),
            1,
            1,
        ));
        let mut output = vec![Vec2::ZERO; 1024];
        let read = source.sample_buffered(&mut output[0..20]);
        assert_eq!(read, 20);
        let read = source.sample_buffered(&mut output[20..80]);
        assert_eq!(read, 60);
        let read = source.sample_buffered(&mut output[80..]);
        assert_eq!(read, 420);

        let expected = (0..500)
            .map(|v| Vec2::splat(v as f32))
            .chain(repeat(Vec2::ZERO))
            .take(1024)
            .collect_vec();

        assert_eq!(output, expected);
    }
}
