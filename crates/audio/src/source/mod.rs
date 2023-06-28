mod buffered;
mod chain;
mod crossfade;
pub(crate) mod dynamic_delay;
pub mod gain;
pub mod history;
mod mix;
mod oscilloscope;
mod pad_to;
mod pan;
mod peek;
mod repeat;
mod sample_bufferer;
mod sample_rate;
mod slice;
mod spatial;
pub mod streaming_source;
mod uniform;
use std::{
    self,
    f32::consts::TAU,
    fmt::Debug,
    ops::{Deref, DerefMut, RangeBounds},
    sync::Arc,
    time::Duration,
};

pub use buffered::*;
pub use chain::*;
use circular_queue::CircularQueue;
pub use crossfade::*;
pub use gain::*;
pub use mix::*;
pub use pan::*;
use parking_lot::Mutex;
pub use peek::*;
pub use repeat::*;
pub use sample_rate::*;
pub use slice::*;
pub use spatial::*;
pub use uniform::*;

use self::{history::History, mix::Mix, oscilloscope::Oscilloscope, pad_to::PadTo};
use crate::{
    blt::{BilinearTransform, Hpf, Lpf, TransferFunction},
    hrtf::HrtfLib,
    value::{Constant, Value},
    AudioEmitter, AudioListener, Frame, SampleRate,
};

/// A source represents a continuous stream of stereo audio samples.
///
/// All sources return a frame of stereo audio.
pub trait Source: Send {
    /// Advance the source to the next sample, returning it.
    ///
    /// When there are no more samples in source, None is returned.
    ///
    /// This is to be considered fused. I.e; after None is returned, Some(Frame) is not expected
    /// to be returned. As such, queue like sources should return Some(0.0) instead of None when
    /// connected but waiting for more sounds to play.
    fn next_sample(&mut self) -> Option<Frame>;

    /// Returns the current sample rate of the source
    fn sample_rate(&self) -> SampleRate;

    /// Returns the number of times `next_sample` will yield
    fn sample_count(&self) -> Option<u64>;

    /// Samples many samples at the same time and adds them to the output
    ///
    /// Returns the number of *Frames* written
    fn sample_buffered(&mut self, output: &mut [Frame]) -> usize {
        output
            .iter_mut()
            .map_while(|v| {
                let sample = self.next_sample()?;

                *v += sample;
                Some(())
            })
            .count()
    }

    /// Returns the duration of this source
    fn duration(&self) -> Option<Duration> {
        Some(Duration::from_nanos(
            (self.sample_count()? * 1_000_000_000) / self.sample_rate(),
        ))
    }

    fn take(self, dur: Duration) -> Slice<Self>
    where
        Self: Sized,
    {
        Slice::new(self, Duration::ZERO..dur)
    }

    fn skip(self, dur: Duration) -> Slice<Self>
    where
        Self: Sized,
    {
        Slice::new(self, dur..)
    }

    fn pad_to(self, dur: Duration) -> PadTo<Self>
    where
        Self: Sized,
    {
        PadTo::new(self, dur)
    }

    fn crossfade<S>(self, other: S, dur: Duration) -> Crossfade<Self, S>
    where
        Self: Sized,
        S: Source + Sized,
    {
        Crossfade::new(self, other, dur)
    }

    fn chain<S>(self, other: S) -> Chain<Self, S>
    where
        Self: Sized,
        S: Source + Sized,
    {
        Chain::new(self, other)
    }

    fn mix<S>(self, other: S) -> Mix<Self, S>
    where
        Self: Sized,
        S: Source + Sized,
    {
        Mix::new(self, other)
    }

    fn slice<R>(self, range: R) -> Slice<Self>
    where
        Self: Sized,
        R: RangeBounds<Duration>,
    {
        Slice::new(self, range)
    }

    fn repeat(self) -> Repeat<Self>
    where
        Self: Sized + Clone,
    {
        Repeat::new(self)
    }

    fn samples_iter(self) -> SampleIter<Self>
    where
        Self: Sized,
    {
        SampleIter::new(self)
    }

    fn gain<G>(self, gain: G) -> Gain<Self, G>
    where
        Self: Sized,
        G: GainValue,
    {
        Gain::new(self, gain)
    }

    fn pan<P>(self, pan: P) -> Pan<Self, P>
    where
        Self: Sized,
        P: PanValue,
    {
        Pan::new(self, pan)
    }

    fn spatial<L, P>(self, hrtf_lib: &HrtfLib, listener: L, params: P) -> Spatial<Self, L, P>
    where
        Self: Sized,
        L: for<'x> Value<'x, Item = AudioListener>,
        P: for<'x> Value<'x, Item = AudioEmitter>,
    {
        Spatial::new(self, hrtf_lib, listener, params)
    }

    fn high_pass(self, freq: f32, bandwidth: f32) -> BilinearTransform<Self, Hpf, Constant<Hpf>>
    where
        Self: Sized,
    {
        BilinearTransform::new(self, Constant(Hpf { freq, bandwidth }))
    }

    fn low_pass(self, freq: f32, bandwidth: f32) -> BilinearTransform<Self, Lpf, Constant<Lpf>>
    where
        Self: Sized,
    {
        BilinearTransform::new(self, Constant(Lpf { freq, bandwidth }))
    }

    fn blt<V, H>(self, transfer: V) -> BilinearTransform<Self, H, V>
    where
        Self: Sized,
        V: for<'x> Value<'x, Item = H>,
        H: Send + Clone + PartialEq + TransferFunction,
    {
        BilinearTransform::new(self, transfer)
    }

    fn history(self, freq: f32, buf: Arc<Mutex<CircularQueue<Frame>>>) -> History<Self>
    where
        Self: Sized,
    {
        History::new(self, freq, buf)
    }

    fn oscilloscope(self, periods: usize, output: Arc<Mutex<Vec<Frame>>>) -> Oscilloscope<Self>
    where
        Self: Sized,
    {
        Oscilloscope::new(self, periods, output)
    }
}

impl<S> Source for Box<S>
where
    S: Source + ?Sized,
{
    #[inline]
    fn next_sample(&mut self) -> Option<Frame> {
        self.deref_mut().next_sample()
    }

    #[inline]
    fn sample_rate(&self) -> SampleRate {
        self.deref().sample_rate()
    }

    #[inline]
    fn sample_count(&self) -> Option<u64> {
        self.deref().sample_count()
    }
}
const DEFAULT_HZ: SampleRate = 44100;

#[derive(Debug, Clone)]
pub struct SineWave {
    freq: f32,
    phase: f32,
    sr: SampleRate,
}

impl SineWave {
    pub fn new(freq: f32) -> Self {
        Self {
            freq,
            phase: 0.0,
            sr: DEFAULT_HZ,
        }
    }
    pub fn phase(mut self, phase: f32) -> Self {
        self.phase = phase;
        self
    }

    pub fn sr(mut self, sr: SampleRate) -> Self {
        self.sr = sr;
        self
    }
}

impl Source for SineWave {
    fn next_sample(&mut self) -> Option<Frame> {
        self.phase += TAU * self.freq / DEFAULT_HZ as f32;
        let v = self.phase.sin();
        Some(Frame::splat(v))
    }

    fn sample_rate(&self) -> SampleRate {
        DEFAULT_HZ
    }

    fn sample_count(&self) -> Option<u64> {
        None
    }

    fn duration(&self) -> Option<Duration> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct SampleIter<S> {
    source: S,
    #[cfg(debug_assertions)]
    max: Option<u64>,
    #[cfg(debug_assertions)]
    count: u64,
}

impl<S> Iterator for SampleIter<S>
where
    S: Source,
{
    type Item = Frame;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.source.next_sample() {
            Some(v) => {
                #[cfg(debug_assertions)]
                {
                    self.count += 1;
                }
                Some(v)
            }
            None => {
                #[cfg(debug_assertions)]
                {
                    debug_assert_eq!(
                        Some(self.count),
                        self.max,
                        "expected source count ({}) to equal max count ({:?})",
                        self.count,
                        self.max
                    );
                }
                None
            }
        }
    }
}

impl<S: Source> SampleIter<S> {
    pub fn new(source: S) -> Self {
        Self {
            #[cfg(debug_assertions)]
            max: source.sample_count(),
            source,
            #[cfg(debug_assertions)]
            count: 0,
        }
    }

    pub fn source(&self) -> &S {
        &self.source
    }
}

pub(crate) fn to_sample_index(sample_rate: SampleRate, time: Duration) -> SampleRate {
    (sample_rate * u64::try_from(time.as_nanos()).unwrap()) / 1_000_000_000
}
