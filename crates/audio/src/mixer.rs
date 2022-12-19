use std::{
    future::Future, sync::{Arc, Weak}, task::Poll, thread, time::Duration
};

use parking_lot::Mutex;
use slotmap::{new_key_type, SlotMap};

use crate::{
    signal::{AsyncSignal, BlockingSignal, Signal}, Frame, SampleConversion, SampleRate, Source
};

new_key_type! {
    pub struct SoundId;
}

type SignalVec = Vec<(SoundId, Arc<dyn Signal>)>;

struct PlayingSound {
    cursor: usize,
    source: Box<dyn Source>,
}

/// Handle to a playing sound
pub struct Sound {
    id: SoundId,
    mixer: AudioMixer,
}

impl Sound {
    /// Wait until the sound finished playing
    pub fn wait(&self) -> SoundFut {
        SoundFut {
            id: self.id,
            signal: None,
            mixer: self.mixer.clone(),
        }
    }

    pub fn wait_blocking(&self) {
        let signal = Arc::new(BlockingSignal::new(thread::current()));
        self.mixer.inner.waiters.lock().push((self.id, signal));
        thread::park()
    }

    pub fn stop(&self) {
        todo!()
    }
}

pub struct SoundFut {
    id: SoundId,
    signal: Option<Arc<AsyncSignal>>,
    mixer: AudioMixer,
}

impl Future for SoundFut {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if let Some(signal) = &self.signal {
            if signal.is_woken() {
                Poll::Ready(())
            } else {
                // Spurious wakeup or poll, replace the waker with the new waker
                signal.set_waker(cx.waker().clone());
                Poll::Pending
            }
        } else {
            // Polled for the first time, place a new waker into the mixer
            let signal = Arc::new(AsyncSignal::new(cx.waker().clone()));

            self.signal = Some(signal.clone());
            self.mixer.inner.waiters.lock().push((self.id, signal));

            Poll::Pending
        }
    }
}

#[derive(Clone, Debug)]
pub struct WeakAudioMixer {
    inner: Weak<AudioMixerInner>,
}

impl WeakAudioMixer {
    pub fn upgrade(&self) -> Option<AudioMixer> {
        Some(AudioMixer {
            inner: self.inner.upgrade()?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct AudioMixer {
    inner: Arc<AudioMixerInner>,
}

struct AudioMixerInner {
    sample_rate: SampleRate,
    waiters: Mutex<SignalVec>,
    sources: Mutex<SlotMap<SoundId, PlayingSound>>,
}

impl std::fmt::Debug for AudioMixerInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioMixerInner")
            .field("sample_rate", &self.sample_rate)
            .field("sources", &self.sources.lock().len())
            .finish()
    }
}

impl AudioMixer {
    pub fn new(sample_rate: SampleRate) -> Self {
        Self {
            inner: Arc::new(AudioMixerInner {
                sample_rate,
                sources: Mutex::default(),
                waiters: Default::default(),
            }),
        }
    }

    pub fn downgrade(&self) -> WeakAudioMixer {
        WeakAudioMixer {
            inner: Arc::downgrade(&self.inner),
        }
    }

    /// Play a source on the mixer, returning a handle which can be used to control it
    pub fn play<S: Source + 'static>(&self, source: S) -> Sound {
        let sample_rate = source.sample_rate();

        let source = if sample_rate == self.inner.sample_rate {
            Box::new(source) as Box<dyn Source>
        } else {
            Box::new(SampleConversion::new(source, self.inner.sample_rate as _)) as Box<dyn Source>
        };

        let id = self
            .inner
            .sources
            .lock()
            .insert(PlayingSound { cursor: 0, source });
        Sound {
            id,
            mixer: self.clone(),
        }
    }

    fn notify_sound_waiters(&self, id: SoundId) {
        // Wake the wakers which are parked on this id, and remove them from the waiting list
        self.inner.waiters.lock().retain_mut(|(sound_id, signal)| {
            if *sound_id == id {
                signal.fire();
                false
            } else {
                true
            }
        })
    }

    #[inline]
    fn terminate_source(&self, id: SoundId, _: &mut PlayingSound) {
        self.notify_sound_waiters(id);
    }
}

impl Source for AudioMixer {
    fn next_sample(&mut self) -> Option<crate::Frame> {
        let mut sources = self.inner.sources.lock();
        let mut res = Frame::ZERO;
        sources.retain(|id, source| {
            let sample = match source.source.next_sample() {
                Some(v) => v,
                None => {
                    self.terminate_source(id, source);
                    return false;
                }
            };
            res += sample;

            true
        });

        Some(res)
    }

    fn sample_rate(&self) -> crate::SampleRate {
        self.inner.sample_rate
    }

    fn sample_buffered(&mut self, output: &mut [Frame]) -> usize {
        let mut sources = self.inner.sources.lock();
        sources.retain(|id, source| {
            let written = source.source.sample_buffered(output);

            // No more samples in source
            if written != output.len() {
                self.terminate_source(id, source);
                return false;
            }

            true
        });

        output.len()
    }

    fn sample_count(&self) -> Option<u64> {
        None
    }
}

impl AudioMixer {
    /// Wait until all audio has stopped playing.
    /// May wait forever on infinite tracks if no timeout is given.
    pub fn wait_idle(&self, timeout: Option<Duration>) {
        let wait = Duration::from_millis(100);
        if let Some(mut timeout) = timeout {
            loop {
                std::thread::sleep(wait.min(timeout));
                timeout = timeout.saturating_sub(wait);
                if self.playing_sinks() == 0 || timeout != Duration::ZERO {
                    break;
                }
            }
        } else {
            loop {
                std::thread::sleep(wait);
                if self.playing_sinks() == 0 {
                    break;
                }
            }
        }
    }

    /// Get the total number of pending tracks for all sinks
    #[must_use]
    pub fn playing_sinks(&self) -> u32 {
        todo!()
        // self.playing_sinks.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// Notify all waiters when dropped
impl Drop for AudioMixerInner {
    fn drop(&mut self) {
        self.waiters.lock().iter_mut().for_each(|(_, v)| v.fire())
    }
}
