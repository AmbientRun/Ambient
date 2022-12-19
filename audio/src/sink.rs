use std::{
    fmt::Debug, sync::{
        atomic::{AtomicU64, Ordering}, Arc
    }, time::Duration
};

use cpal::{ChannelCount, Sample};
use flume::{Receiver, Sender, TryRecvError};
use glam::Vec3;

use crate::{
    spatial::{AudioListener, SpatialParams}, BoxSourceInit, NanoTime, SinkId, SourceInit, StreamConfig
};

pub const SPEED_OF_SOUND: f32 = 343.0; // m/s

#[derive(Debug, Clone)]
pub(crate) enum SinkEvent {
    /// Skip the current track
    Skip,
    /// Skip n items in the queue, and only the queue
    ClearQueue(u16),
    Pause(bool),
    Mute(bool),
}

#[derive(Debug)]
pub(crate) struct SinkData {
    // LR
    volume: AtomicU64,
    // LR
    delay: AtomicU64,
}

impl Default for SinkData {
    fn default() -> Self {
        Self {
            volume: AtomicU64::new(((1.0_f32.to_bits() as u64) << 32) | (1.0_f32.to_bits() as u64)),
            delay: AtomicU64::new(0),
        }
    }
}

impl SinkData {
    /// Sets the left and right sink volume
    pub fn set_volume(&self, l: f32, r: f32) {
        self.volume.store(
            (l.to_bits() as u64) << 32 | r.to_bits() as u64,
            Ordering::Relaxed,
        );
    }

    pub fn get_volume(&self) -> (f32, f32) {
        let bits = self.volume.load(Ordering::Relaxed);

        (
            f32::from_bits((bits >> 32) as u32),
            f32::from_bits(bits as u32),
        )
    }

    /// Sets the left and right sink sample delay in microseconds
    pub fn set_delay(&self, l: u32, r: u32) {
        // Convert to micros and store.
        // This can store > 4 seconds of delay and also provides a fidelity
        // greater than common sample rates.
        self.delay
            .store((l as u64) << 32 | (r as u64), Ordering::Relaxed);
    }

    /// Returns sink delay in micros
    pub fn get_delay_raw(&self) -> (u32, u32) {
        let bits = self.delay.load(Ordering::Relaxed);

        ((bits >> 32) as u32, bits as u32)
    }

    #[allow(dead_code)]
    pub fn get_delay(&self) -> (Duration, Duration) {
        let (l, r) = self.get_delay_raw();
        (Duration::from_micros(l as _), Duration::from_micros(r as _))
    }
}

pub type BoxCb = Box<dyn FnOnce(SinkId) + Send + Sync>;

/// Holds a initialized source and an *optional* callback for when the source
/// finishes playing.
///
/// `on_finish` is also called when the source is skipped or stopped.
pub struct Payload {
    pub source: BoxSourceInit,
    pub on_finish: Option<BoxCb>,
}

impl Debug for Payload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Payload")
            .field("source", &self.source)
            .finish()
    }
}

impl Payload {
    pub fn new(
        source: BoxSourceInit,
        on_finish: impl FnOnce(SinkId) + Send + 'static + Sync,
    ) -> Self {
        Self {
            source,
            on_finish: Some(Box::new(on_finish)),
        }
    }

    pub fn from_source(source: BoxSourceInit) -> Self {
        Self {
            source,
            on_finish: None,
        }
    }
}

impl From<BoxSourceInit> for Payload {
    fn from(v: BoxSourceInit) -> Self {
        Self::from_source(v)
    }
}

#[derive(Debug)]
/// Sink for playing sounds.
/// Dropping a sink will stop the audio and remove the sink from the internal
/// mixer
pub struct Sink {
    queue: Sender<Payload>,
    events: Sender<SinkEvent>,
    id: SinkId,
    data: Arc<SinkData>,

    paused: bool,
    muted: bool,
}

impl Sink {
    pub(crate) fn new(
        queue: Sender<Payload>,
        events: Sender<SinkEvent>,
        id: SinkId,
        data: Arc<SinkData>,
    ) -> Self {
        Self {
            queue,
            events,
            id,
            data,
            paused: false,
            muted: false,
        }
    }

    pub fn play_now(&self, source: impl Into<Payload>) -> &Self {
        // Skip all pending tracks
        self.stop().enqueue(source);
        self
    }

    /// Stops playback of the sink and empties the queue
    pub fn stop(&self) -> &Self {
        self.events.send(SinkEvent::Skip).unwrap();
        self.events
            .send(SinkEvent::ClearQueue(
                self.queue
                    .len()
                    .try_into()
                    .expect("Too many items in the queue"),
            ))
            .unwrap();
        self
    }

    pub fn enqueue(&self, source: impl Into<Payload>) -> &Self {
        self.queue
            .send(source.into())
            .expect("Failed to queue source");
        self
    }

    /// Attempts to queue a track. Fails if the queue is full
    pub fn try_enqueue(
        &self,
        source: impl Into<Payload>,
    ) -> std::result::Result<&Self, flume::TrySendError<Payload>> {
        self.queue.try_send(source.into())?;
        Ok(self)
    }

    /// Returns the current queue length
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn id(&self) -> SinkId {
        self.id
    }

    /// Sets the sink volume
    ///
    /// Default is 1.0, 1.0
    pub fn set_volume(&self, l: f32, r: f32) -> &Self {
        self.data.set_volume(l, r);
        self
    }

    /// Sets the delay between the current time and the time of the audio.
    pub fn set_delay(&self, l: Duration, r: Duration) -> &Self {
        self.data.set_delay(l.as_micros() as _, r.as_micros() as _);
        self
    }

    pub fn set_delay_secs(&self, l: f32, r: f32) -> &Self {
        self.set_delay(Duration::from_secs_f32(l), Duration::from_secs_f32(r));
        self
    }

    pub fn get_volume(&self) -> (f32, f32) {
        self.data.get_volume()
    }

    /// Mute the sink.
    ///
    /// This is unlike setting the volume to 0.
    /// Useful for muting sinks which have a volume set by spatial audio
    /// systems.
    pub fn mute(&mut self, mute: bool) -> &mut Self {
        if mute != self.muted {
            self.events.try_send(SinkEvent::Mute(mute)).unwrap();
            self.muted = mute;
        }
        self
    }

    /// Pauses the sink
    pub fn pause(&mut self, pause: bool) -> &mut Self {
        if pause != self.paused {
            self.events.try_send(SinkEvent::Pause(pause)).unwrap();
            self.paused = pause;
        }
        self
    }

    /// Sets the sink volume and delay from spatial information
    pub fn set_spatial(
        &self,
        pos: Vec3,
        params: &SpatialParams,
        listener: &AudioListener,
    ) -> &Self {
        let dist_l = pos.distance(listener.left);
        let dist_r = pos.distance(listener.right);
        // tracing::info!(dist_l, dist_r);
        let vol_l = params.amplitude * params.attn.attenuate(dist_l);
        let vol_r = params.amplitude * params.attn.attenuate(dist_r);

        let delay_l = dist_l / SPEED_OF_SOUND;
        let delay_r = dist_r / SPEED_OF_SOUND;

        self.set_volume(vol_l, vol_r)
            .set_delay_secs(delay_l, delay_r);

        self
    }

    pub fn muted(&self) -> bool {
        self.muted
    }

    pub fn paused(&self) -> bool {
        self.paused
    }
}

/// The receiving end of the sink which plays the audio
#[derive(Debug)]
pub(crate) struct SinkConsumer {
    queue: Receiver<Payload>,
    data: Arc<SinkData>,

    event_rx: Receiver<SinkEvent>,

    paused: bool,
    muted: bool,

    current: Option<Payload>,

    /// The position into the track
    cursor: u64,
}

impl SinkConsumer {
    pub(crate) fn new(
        queue: Receiver<Payload>,
        event_rx: Receiver<SinkEvent>,
        data: Arc<SinkData>,
    ) -> Self {
        Self {
            queue,
            data,
            current: None,
            cursor: 0,
            paused: false,
            muted: false,
            event_rx,
        }
    }

    pub fn write<T: Sample>(
        &mut self,
        id: SinkId,
        data: &mut [T],
        config: &StreamConfig,
    ) -> SinkState {
        // Handle incoming operations
        for event in self.event_rx.drain() {
            match event {
                SinkEvent::Skip => {
                    if let Some(current) = self.current.take() {
                        if let Some(callback) = current.on_finish {
                            (callback)(id)
                        }
                    }
                }
                SinkEvent::ClearQueue(count) => {
                    for item in self.queue.try_iter().take(count as _) {
                        if let Some(callback) = item.on_finish {
                            (callback)(id)
                        }
                    }
                }
                SinkEvent::Pause(pause) => self.paused = pause,
                SinkEvent::Mute(mute) => self.muted = mute,
            }
        }

        if self.paused {
            return SinkState::Pending;
        } else if self.muted {
            let (delay_l, delay_r) = self.data.get_delay_raw();
            // Transform delay to a sample count
            let delay = [
                config.to_sample(NanoTime::from_micros(delay_l as _)),
                config.to_sample(NanoTime::from_micros(delay_r as _)),
            ];

            if let Err(e) =
                self.get_sample(id, delay[0], data.len() as _, config.channels - 1, config)
            {
                return e;
            };

            return SinkState::Playing;
        }

        let mut prev_delay = Default::default();
        for (i, sample) in data.iter_mut().enumerate() {
            // Move inside loop since this function is called around 10
            // times/second, and audio volume may change often than that
            let (vol_l, vol_r) = self.data.get_volume();
            let (delay_l, delay_r) = self.data.get_delay_raw();
            // Transform delay to a sample count
            let delay = [
                config.to_sample(NanoTime::from_micros(delay_l as _)),
                config.to_sample(NanoTime::from_micros(delay_r as _)),
            ];

            // if delay != prev_delay {
            //     tracing::info!(?vol_l, ?vol_r, ?delay_l, ?delay_r, ?delay);
            // }

            let volume = [vol_l, vol_r];
            prev_delay = delay;
            // log::info!("{delay_l}:{delay_r} {delay:?}");
            // log::info!("Vol: {vol_l}:{vol_r}");
            let channel = (i as u16) % config.channels;

            let v = match self.get_sample(id, delay[channel as usize], 1, channel, config) {
                Ok(v) => v,
                Err(e) => return e,
            };

            *sample = Sample::from(&(sample.to_f32() + (v * volume[channel as usize])));
        }

        SinkState::Playing
    }

    fn acquire_current(&mut self) -> Result<(&mut Payload, &mut u64), SinkState> {
        match self.current {
            Some(ref mut current) => Ok((current, &mut self.cursor)),
            None => {
                self.cursor = 0;
                match self.queue.try_recv() {
                    Ok(next) => Ok((self.current.insert(next), &mut self.cursor)),
                    Err(TryRecvError::Empty) => Err(SinkState::Pending),
                    Err(TryRecvError::Disconnected) => Err(SinkState::Disconnected),
                }
            }
        }
    }

    /// Returns the sample at the current position and steps
    pub fn get_sample(
        &mut self,
        id: SinkId,
        delay: u64,
        stride: u64,
        channel: ChannelCount,
        config: &StreamConfig,
    ) -> Result<f32, SinkState> {
        // Repeatedly fetch next sample.
        // Usually runs once, though may run multiple times if the current
        // source ends and a new one needs to be loaded
        loop {
            let (current, cursor) = self.acquire_current()?;

            let pos = cursor.saturating_sub(delay);
            if channel == 0 {
                log::info!("Playing sample: d: {delay} {pos}");
            }
            match current.source.at(pos, channel, config) {
                Some(v) => {
                    *cursor += (channel == (config.channels - 1)) as u64 * stride;
                    return Ok(v);
                }
                None => {
                    // This track is done. To not skip a sample, fetch again
                    if let Some(callback) = self.current.take().unwrap().on_finish {
                        (callback)(id)
                    }
                }
            }
        }

        // let mut taken = 0;
        // loop {
        //     let current = match self.current {
        //         Some(ref mut current) => Ok(current),
        //         None => match self.rx.try_recv() {
        //             Ok(next) => {
        //                 self.sample = 0;
        //                 Ok(self.current.insert(next))
        //             }
        //             Err(TryRecvError::Empty) => Err(SinkState::Pending),
        //             Err(TryRecvError::Disconnected) => Err(SinkState::Disconnected),
        //         },
        //     }?;

        //     loop {
        //         match current.source.at(self.sample, channel, config) {
        //             Some(v) => {
        //                 taken+=1;

        //             },
        //             None => break,
        //         }
        //     }
        // }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SinkState {
    Playing,
    /// Sink is still connected, but no data at the moment.
    Pending,
    /// Sink is not connected and should be removed.
    Disconnected,
}
