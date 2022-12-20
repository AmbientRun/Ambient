use std::ops::AddAssign;

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait}, OutputCallbackInfo, Sample
};
use itertools::Itertools;
use slotmap::new_key_type;

use crate::{
    error::{Error, Result}, AudioMixer, ChannelCount, Frame, SampleRate, Source, WeakAudioMixer
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StreamConfig {
    pub sample_rate: SampleRate,
    pub channels: ChannelCount,
}

// impl StreamConfig {
//     pub fn to_ns(&self, sample: u64) -> NanoTime {
//         NanoTime::from((sample * NANOSECS) / self.sample_rate)
//     }

//     pub fn to_sample(&self, ns: NanoTime) -> u64 {
//         (ns.as_nanos() * self.sample_rate) / NANOSECS
//     }

//     pub fn to_sample_f32(&self, ns: NanoTime) -> f32 {
//         (ns.as_f32() * self.sample_rate as f32) / NANOSECS as f32
//     }
// }

impl From<cpal::StreamConfig> for StreamConfig {
    fn from(v: cpal::StreamConfig) -> Self {
        Self {
            sample_rate: v.sample_rate.0 as _,
            channels: v.channels,
        }
    }
}

/// Wraps a cpal Stream. Can not be moved across threads.
pub struct AudioStream {
    _stream: cpal::Stream,
    mixer: AudioMixer,
    _device: cpal::Device,
}

impl AudioStream {
    pub fn new() -> Result<Self> {
        let device = cpal::default_host()
            .default_output_device()
            .ok_or(Error::NoOutputDevice)?;

        let config = device.default_output_config()?;
        // .ok()
        // .or_else(|| Some(device.supported_output_configs().ok()?.next()?.with_max_sample_rate()))
        // .ok_or(Error::NoOutputConfig)?;

        let format = config.sample_format();
        let config: cpal::StreamConfig = config.into();

        tracing::info!("Audio stream config: {config:?}");
        if config.channels < 1 || config.channels > 2 {
            return Err(Error::InvalidChannelCount(config.channels));
        }

        let mixer_config: crate::StreamConfig = config.clone().into();

        let mixer = AudioMixer::new(mixer_config.sample_rate);

        let weak_mixer = mixer.downgrade();

        let err_func = |err| log::error!("Audio error: {err}");

        let channels = mixer_config.channels;

        fn writer<T>(
            mixer: WeakAudioMixer,
            channel_count: u16,
        ) -> impl FnMut(&mut [T], &OutputCallbackInfo)
        where
            T: Sample + AddAssign<T>,
        {
            let mut buf = Vec::new();
            move |data, _| {
                buf.resize(data.len() / channel_count as usize, Frame::ZERO);

                for v in &mut buf {
                    *v = Frame::ZERO;
                }

                if let Some(mut mixer) = mixer.upgrade() {
                    mixer.sample_buffered(&mut buf);
                }

                // Write to the concrete type buffer
                if channel_count == 1 {
                    for (dst, src) in data.iter_mut().zip_eq(&buf) {
                        *dst = T::from(&((src.x + src.y) / 2.0));
                    }
                } else if channel_count == 2 {
                    for ((l, r), src) in data.iter_mut().tuples().zip_eq(&buf) {
                        *l = T::from(&src.x);
                        *r = T::from(&src.y);
                    }
                } else {
                    unimplemented!()
                }
            }
        }

        let stream = match format {
            cpal::SampleFormat::I16 => {
                device.build_output_stream(&config, writer::<i16>(weak_mixer, channels), err_func)
            }
            cpal::SampleFormat::U16 => {
                device.build_output_stream(&config, writer::<u16>(weak_mixer, channels), err_func)
            }
            cpal::SampleFormat::F32 => {
                device.build_output_stream(&config, writer::<f32>(weak_mixer, channels), err_func)
            }
        }?;

        stream.play()?;

        Ok(Self {
            mixer,
            _stream: stream,
            _device: device,
        })
    }

    /// Get a reference to the audio stream's mixer.
    #[must_use]
    pub fn mixer(&self) -> &AudioMixer {
        &self.mixer
    }
}

new_key_type! {
    /// Represents the id of the sink
    pub struct SinkId;
}
