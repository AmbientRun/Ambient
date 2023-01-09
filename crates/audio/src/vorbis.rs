use std::{io::Cursor, sync::Arc};

use derivative::Derivative;
use itertools::Itertools;
use lewton::inside_ogg::OggStreamReader;

use crate::{ChannelCount, Error, Frame, Result, SampleRate, Source};

/// A packet of multi-channel interleaved samples
struct FramedSamples {
    pub samples: Result<Vec<Frame>>,
    pub channel_count: usize,
}

impl lewton::samples::Samples for FramedSamples {
    fn num_samples(&self) -> usize {
        if let Ok(samples) = &self.samples {
            samples.len() / self.channel_count
        } else {
            0
        }
    }

    fn truncate(&mut self, limit: usize) {
        if let Ok(samples) = &mut self.samples {
            samples.truncate(limit * self.channel_count);
        }
    }

    fn from_floats(floats: Vec<Vec<f32>>) -> Self {
        let channel_count = floats.len();
        // Note that a channel count of 0 is forbidden
        // by the spec and the header decoding code already
        // checks for that.
        assert!(!floats.is_empty());

        let samples: Result<Vec<Frame>> = if channel_count == 1 {
            Ok(floats[0].iter().copied().map(Frame::splat).collect_vec())
        } else if channel_count == 2 {
            Ok(floats[0]
                .iter()
                .copied()
                .zip_eq(floats[1].iter().copied())
                .map_into()
                .collect_vec())
        } else {
            Err(Error::TooManyOggChannels(channel_count))
        };

        Self {
            samples,
            channel_count,
        }
    }
}

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct VorbisTrack {
    #[derivative(Debug = "ignore")]
    bytes: Arc<[u8]>,
    /// The number of frames in the fully decoded audio
    decoded_len: usize,
}

impl VorbisTrack {
    pub fn new(bytes: impl Into<Arc<[u8]>>) -> Result<Self> {
        // Decode once to ensure there are no encoding errors, and acquire the duration

        let bytes = bytes.into();
        let mut streamer = OggStreamReader::new(Cursor::new(&bytes[..]))?;

        let channels: ChannelCount = streamer.ident_hdr.audio_channels as _;

        let mut len = 0;
        while let Some(s) = streamer.read_dec_packet_generic::<FramedSamples>()? {
            let samples = s.samples?;
            assert_eq!(
                samples.len() % channels as usize,
                0,
                "Expected packet size to be evenly divisible by channel count"
            );
            debug_assert_eq!(channels as usize, s.channel_count);

            len += samples.len() / channels as usize;
        }

        Ok(Self {
            bytes,
            decoded_len: len,
        })
    }

    pub fn decode(&self) -> VorbisDecodeStream {
        let streamer = OggStreamReader::new(Cursor::new(self.bytes.clone())).unwrap();
        let _channels: ChannelCount = streamer.ident_hdr.audio_channels as _;
        let _sample_rate: SampleRate = streamer.ident_hdr.audio_sample_rate as _;

        VorbisDecodeStream {
            streamer,
            decoded_len: self.decoded_len,
            packet: Vec::new(),
            cursor: 0,
            bytes: self.bytes.clone(),
        }
    }
}

/// Audio source which decodes a compressed ogg stream
pub struct VorbisDecodeStream {
    bytes: Arc<[u8]>,
    streamer: OggStreamReader<Cursor<Arc<[u8]>>>,
    decoded_len: usize,
    packet: Vec<Frame>,
    cursor: usize,
}

impl Clone for VorbisDecodeStream {
    fn clone(&self) -> Self {
        Self {
            streamer: OggStreamReader::new(Cursor::new(self.bytes.clone())).unwrap(),
            bytes: self.bytes.clone(),
            decoded_len: self.decoded_len,
            packet: Vec::new(),
            cursor: 0,
        }
    }
}

impl Source for VorbisDecodeStream {
    #[inline]
    fn next_sample(&mut self) -> Option<crate::Frame> {
        if let Some(&s) = self.packet.get(self.cursor) {
            self.cursor += 1;
            Some(s)
        } else {
            // Read the next packet
            // Creation of the Track ensures decoding works
            loop {
                let pkt = self
                    .streamer
                    .read_dec_packet_generic::<FramedSamples>()
                    .unwrap()?;

                self.packet = pkt.samples.unwrap();

                if let Some(&s) = self.packet.get(0) {
                    self.cursor = 1;
                    return Some(s);
                }
            }
        }
    }

    fn sample_rate(&self) -> SampleRate {
        self.streamer.ident_hdr.audio_sample_rate as _
    }

    fn sample_count(&self) -> Option<u64> {
        Some(self.decoded_len as _)
    }
}
