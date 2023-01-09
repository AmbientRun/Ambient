use crate::{Frame, Peek, SampleRate, Source};

/// A source which converts another source's sample rate and channel count to another.
///
/// This conversion uses a *physically accurate* conversion. That is, the energy of the signal
/// waveform from the source is conserved.
///
/// If the source sample rate is greater than the destination, the last sample in the source will
/// be stretched to a duration of the destination sample rate. This does *not* preserve the energy,
/// as an energy preserving method would be to average the last sample with silence, but since it
/// is not known what follows, the "incorrect" method fixes the odd quiet sample between tracks,
/// and allow for gapless playback.
#[derive(Debug, Clone)]
pub struct SampleConversion<S> {
    src_rate: SampleRate,
    dst_rate: SampleRate,

    /// If the src_rate is low than dst_rate, this points to the fractional parts of the src
    /// samples.
    ///
    /// E.g, with a sample ratio of `2/3`, it will advance by `2` each time `next_sample` is
    /// called.
    ///
    /// 0, 2, 1, 0, 2, 1.
    ///
    /// The previous and next sample will be linearly interpolate by the subsample
    ///
    /// If src_rate is greater than dst_rate, it will advance by more than a sample
    subsample: SampleRate,

    /// This is None iff the source was completely empty from the beginning
    cur_dst_frame: Option<Frame>,
    next_dst_frame: Option<Frame>,

    source: Peek<S>,
}

impl<S: Source> SampleConversion<S> {
    pub fn new(source: S, dst_rate: SampleRate) -> Self {
        let src_rate = source.sample_rate();

        let mut source = Peek::new(source);

        // This steps at least one frame in source, but if dst_rate is lower, it will step multiple
        let step = src_rate.max(dst_rate);

        let (first_frame, start) = Self::consume_frames(step, dst_rate, &mut source);
        let _dst = src_rate + src_rate.max(dst_rate);

        let (second_frame, start) = Self::consume_frames(start + step, dst_rate, &mut source);

        Self {
            src_rate,
            dst_rate,
            source,
            /// Start the subsample at the end of the before-beginning sample.
            ///
            /// This forces the first call to `next_sample` to fetch the first sample, and lerp it
            /// by 1.0
            subsample: start,

            cur_dst_frame: first_frame,
            next_dst_frame: second_frame,
        }
    }

    /// Consumes frames from the source from start..end, and returns their average.
    ///
    /// If start is in between two frames the value is linearly interpolated
    pub fn consume_frames(
        mut cursor: SampleRate,
        step: SampleRate,
        source: &mut Peek<S>,
    ) -> (Option<Frame>, SampleRate) {
        let mut acc = Frame::default();
        let mut steps = 0;

        while cursor >= step {
            let current = source.next_sample();
            // eprintln!("Advancing source to {current:?}");
            let current = match current {
                Some(v) => v,
                None => break,
            };

            // Since we may need to lerp
            //
            // *Pretend* that the last sample goes on forever
            let _next = source.peek().unwrap_or(current);

            // Position within the sample
            let t = (cursor % step) as f32 / step as f32;

            assert!(t < 1.0);
            let value = current;

            acc += value;

            cursor -= step;
            steps += 1;
        }

        cursor %= step;
        // There were no more, not even truncated frames, in the source
        if steps == 0 {
            return (None, cursor);
        }

        let value = acc / steps as f32;

        (Some(value), cursor)
    }

    pub fn dst_rate(&self) -> u64 {
        self.dst_rate
    }
}

impl<S: Source> Source for SampleConversion<S> {
    #[inline(always)]
    fn next_sample(&mut self) -> Option<Frame> {
        // The current subsample is beyond a dst_sample. This means we need to step one or more
        // samples forward in the source.
        //
        // Multiple steps are taken since the src_rate is higher, in which case each dst_sample
        // will average multiple src_samples, rather than discarding them
        if self.subsample >= self.dst_rate {
            // eprintln!("new_frame");
            // New frame?
            let (next, new_cursor) =
                Self::consume_frames(self.subsample, self.dst_rate, &mut self.source);
            // eprintln!("Acquired new frame: {next:?} ending at: {new_cursor}");

            // Slide the two dst frames we are interpolating over
            self.cur_dst_frame = Some(self.next_dst_frame?);
            self.next_dst_frame = next;
            self.subsample = new_cursor;
        }

        let cur = self.cur_dst_frame?;
        let next = self.next_dst_frame.unwrap_or(cur);

        // Linearly interpolate this dst frame with the next one
        let t = (self.subsample % self.dst_rate) as f32 / self.dst_rate as f32;

        let value = cur + t * (next - cur);

        // Advance by the other sample rate.
        //
        //
        // This has the effect of tracking where in the src samples we are currently
        // When this cursor is past the src_cursor, we stepped over a sample, and need to
        // keep up with the src_cursor.
        self.subsample += self.src_rate;

        Some(value)
    }

    fn sample_rate(&self) -> SampleRate {
        self.dst_rate
    }

    fn sample_count(&self) -> Option<u64> {
        let samples = self.source.sample_count()?;
        let samples = div_ceil(samples * self.dst_rate, self.src_rate);
        Some(samples)
    }
}

fn div_ceil(a: u64, b: u64) -> u64 {
    dbg!(a, b);
    (a + b - 1) / b
}

#[cfg(test)]
mod test {

    use std::sync::Arc;

    use itertools::Itertools;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{BufferedSource, SampleIter};

    #[test]
    fn buffered() {
        let buf: Arc<[f32]> = Arc::from([0.0, 0.1, 1.0, 1.1, 2.0, 2.1, 1.0, 1.1, 0.0, 0.1]);

        let samples = BufferedSource::new(buf, 2, 4).samples_iter().collect_vec();

        assert_eq!(
            samples,
            [
                Frame::new(0.0, 0.1),
                Frame::new(1.0, 1.1),
                Frame::new(2.0, 2.1),
                Frame::new(1.0, 1.1),
                Frame::new(0.0, 0.1),
            ]
        );
    }

    #[test]
    fn sample_identity() {
        let buf: Arc<[f32]> = Arc::from([0.0, 0.0, 1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0]);

        let source = SampleConversion::new(BufferedSource::new(buf, 2, 6), 6);
        assert_eq!(source.sample_rate(), 6);
        assert_eq!(source.sample_count(), Some(5));

        let samples = SampleIter::new(source).collect_vec();

        dbg!(&samples);
        assert_eq!(
            samples,
            [
                Frame::new(0.0, 0.0),
                Frame::new(1.0, 1.0),
                Frame::new(2.0, 2.0),
                Frame::new(3.0, 3.0),
                Frame::new(4.0, 4.0)
            ]
        );
    }

    #[test]
    /// Upscales a source by a factor of two.
    ///
    /// This causes samples in between the source samples to be generated
    fn upscale() {
        let buf: Arc<[f32]> = Arc::from([0.0, 0.0, 1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0]);
        let source = SampleConversion::new(BufferedSource::new(buf, 2, 2), 4);

        assert_eq!(source.sample_count(), Some(10));
        let samples = SampleIter::new(source).collect_vec();
        // |----second-----|
        // 0.0     1.0     2.0     3.0     4.0
        // *-------*-------*-------*-------*-------   2 Hz
        // *---*---*---*---*---*---*---*---*---*---   4 Hz

        dbg!(&samples);
        #[rustfmt::skip]
        assert_eq!(
            samples,
            [
                Frame::new(0.0, 0.0),
                Frame::new(0.5, 0.5),
                Frame::new(1.0, 1.0),
                Frame::new(1.5, 1.5),
                Frame::new(2.0, 2.0),
                Frame::new(2.5, 2.5),
                Frame::new(3.0, 3.0),
                Frame::new(3.5, 3.5),
                Frame::new(4.0, 4.0),
                Frame::new(4.0, 4.0)
            ]
        );
    }

    #[test]
    /// Upscales a source by a factor of two.
    ///
    /// This causes samples in between the source samples to be generated
    fn upscale_uneven() {
        let buf: Arc<[f32]> = Arc::from([
            0.0, 0.0, 1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0, 5.0, 5.0, 6.0, 6.0,
        ]);
        let source = SampleConversion::new(BufferedSource::new(buf, 2, 2), 3);

        // |----second-------|
        // 0.0      1.0      2.0      3.0      4.0      5.0      6.0
        // *--------*--------*--------*--------*--------*--------*-------- 2 Hz
        // *-----*-----*-----*-----*-----*-----*-----*-----*-----*-----*----- 3 Hz
        // 0.0   0.67 1.33  2.0   2.67  3.33   4.0   4.67  5.33  6.0
        assert_eq!(source.sample_count(), Some(11));

        let samples = SampleIter::new(source)
            .map(|v| (v * 100.0).round() / 100.0)
            .collect_vec();

        dbg!(&samples);
        #[rustfmt::skip]
        assert_eq!(
            samples,
            [
                Frame::new(0.0,  0.0),
                Frame::new(0.67, 0.67),
                Frame::new(1.33, 1.33),
                Frame::new(2.0,  2.0),
                Frame::new(2.67, 2.67),
                Frame::new(3.33, 3.33),
                Frame::new(4.0,  4.0),
                Frame::new(4.67, 4.67),
                Frame::new(5.33, 5.33),
                Frame::new(6.0,  6.0),
                Frame::new(6.0,  6.0),
            ]
        );
    }

    #[test]
    /// Downscale a source.
    ///
    /// This causes the range of samples from i to i+step to be averaged into a single value for i
    fn downscale() {
        let buf: Arc<[f32]> = Arc::from([
            0.0, 0.0, 0.5, 0.5, 1.0, 1.0, 1.5, 1.5, 2.0, 2.0, 2.5, 2.5, 3.0, 3.0, 3.5, 3.5, 4.0,
            4.0, 4.5, 4.5, 5.0, 5.0, 5.5, 5.5, 6.0, 6.0,
        ]);

        let source = SampleConversion::new(BufferedSource::new(buf, 2, 6), 2);

        assert_eq!(source.sample_count(), Some(5));

        // |--------------second---------------|
        // 0.0   0.5   1.0   1.5   2.0   2.5   3.0   3.5   4.0   4.5   5.0   5.5   6.0
        // *-----*-----*-----*-----*-----*-----*-----*-----*-----*-----*-----*-----* 6 Hz
        // 0     2     4     6     8     10    12    14    16    18    20    22    24
        // *-----------------*-----------------*-----------------*-----------------* 2 Hz
        // 0.5               2.0               3.5               5.0
        // ^-----------------
        // The returned PCM data for sample [0] will be played for the whole duration between value
        // 0.0->1.5.
        //
        // As such, the returned sample needs to be the forward averaged up until, but not
        // including, the next sample.

        let samples = SampleIter::new(source)
            .map(|v| (v * 100.0).round() / 100.0)
            .collect_vec();

        dbg!(&samples);
        assert_eq!(
            samples,
            [
                Frame::new(0.5, 0.5),
                Frame::new(2.0, 2.0),
                Frame::new(3.5, 3.5),
                Frame::new(5.0, 5.0),
                Frame::new(6.0, 6.0)
            ]
        );
    }

    #[test]
    /// Downscale a source.
    ///
    /// This causes the range of samples from i to i+step to be averaged into a single value for i
    fn downscale2() {
        let buf: Arc<[f32]> = Arc::from([
            2.0, 2.0, 3.0, 3.0, 4.0, 4.0, 5.0, 5.0, 5.0, 5.0, 4.5, 4.5, 3.0, 3.0, 2.0, 2.0,
        ]);

        let source = SampleConversion::new(BufferedSource::new(buf, 2, 6), 2);
        assert_eq!(source.sample_count(), Some(3));

        // |--------------second---------------|
        // 2.0   3.0   4.0   5.0   5.0   4.5   3.0   2.0
        // *-----*-----*-----*-----*-----*-----*-----*-----*-----*-----*-----*-----* 6 Hz
        // 0     2     4     6     8     10    12    14    16    18    20    22    24
        // *-----------------*-----------------*-----------------*-----------------* 2 Hz
        // 3.0               4.83              2.5
        // ^-----------------
        // The returned PCM data for sample [0] will be played for the whole duration between value
        // 3.0->4.83.
        //
        // As such, the returned sample needs to be the forward averaged up until, but not
        // including, the next sample.

        let samples = SampleIter::new(source)
            .map(|v| (v * 100.0).round() / 100.0)
            .collect_vec();

        dbg!(&samples);
        assert_eq!(
            samples,
            [
                Frame::new(3.0, 3.0),
                Frame::new(4.83, 4.83),
                Frame::new(2.5, 2.5)
            ]
        );
    }

    #[test]
    pub fn empty_source() {
        let buf: Arc<[f32]> = Arc::from([]);
        let source = SampleConversion::new(BufferedSource::new(buf, 2, 2), 6);

        assert_eq!(SampleIter::new(source).collect_vec(), &[]);
    }
}
