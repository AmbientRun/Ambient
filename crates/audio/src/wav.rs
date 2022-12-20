use std::{io::Cursor, sync::Arc};

use cpal::{ChannelCount, Sample};
use derivative::Derivative;
use glam::{vec2, Vec2};
use hound::{SampleFormat, WavReader, WavSpec};
use itertools::Itertools;

use crate::{Frame, Result, SampleRate, Source};

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct WavTrack {
    #[derivative(Debug = "ignore")]
    bytes: Arc<[u8]>,
    /// The number of frames in the fully decoded audio
    decoded_len: usize,
}

impl WavTrack {
    pub fn new(bytes: Arc<[u8]>) -> Result<Self> {
        let mut reader = WavReader::new(Cursor::new(&bytes[..]))?;

        let WavSpec {
            channels,
            bits_per_sample,
            sample_format,
            ..
        } = reader.spec();

        // Convert samples as appropriate
        let samples = match (sample_format, bits_per_sample) {
            (SampleFormat::Int, 16) => reader
                .samples::<i16>()
                .map_ok(|_| ())
                .collect::<std::result::Result<Arc<[_]>, _>>(),
            (SampleFormat::Int, 24) => reader
                .samples::<i32>()
                .map_ok(|_| ())
                .collect::<std::result::Result<Arc<_>, _>>(),
            (SampleFormat::Int, 32) => reader
                .samples::<i32>()
                .map_ok(|_| ())
                .collect::<std::result::Result<Arc<_>, _>>(),
            (SampleFormat::Float, 32) => reader
                .samples::<f32>()
                .map_ok(|_| ())
                .collect::<std::result::Result<Arc<_>, _>>(),
            _ => {
                panic!("Unsupported wav format")
            }
        }?;

        let decoded_len = samples.len() / channels as usize;

        Ok(Self { bytes, decoded_len })
    }

    pub fn decode(&self) -> WavDecodeStream {
        let streamer = WavReader::new(Cursor::new(self.bytes.clone())).unwrap();

        let WavSpec {
            channels,
            sample_rate,
            bits_per_sample,
            sample_format,
        } = streamer.spec();

        WavDecodeStream {
            streamer,
            decoded_len: self.decoded_len,
            cursor: 0,
            channels,
            format: sample_format,
            bits_per_sample,
            current_block: Vec::new(),
            sample_rate: sample_rate as _,
            bytes: self.bytes.clone(),
        }
    }
}

pub const WAV_BLOCK_SIZE: usize = 512;

pub struct WavDecodeStream {
    bytes: Arc<[u8]>,
    channels: ChannelCount,
    streamer: WavReader<Cursor<Arc<[u8]>>>,
    format: SampleFormat,
    bits_per_sample: u16,

    sample_rate: SampleRate,
    current_block: Vec<Frame>,
    cursor: usize,
    decoded_len: usize,
}

impl Clone for WavDecodeStream {
    fn clone(&self) -> Self {
        Self {
            bytes: self.bytes.clone(),
            streamer: WavReader::new(Cursor::new(self.bytes.clone())).unwrap(),
            channels: self.channels,
            format: self.format,
            bits_per_sample: self.bits_per_sample,
            sample_rate: self.sample_rate,
            current_block: Vec::new(),
            cursor: 0,
            decoded_len: self.decoded_len,
        }
    }
}

fn append_frames(
    iter: impl Iterator<Item = std::result::Result<f32, hound::Error>>,
    res: &mut Vec<Frame>,
    channels: ChannelCount,
) -> Result<()> {
    if channels == 1 {
        for v in iter.take(WAV_BLOCK_SIZE) {
            let v = v?;
            res.push(Vec2::splat(v));
        }
    } else if channels == 2 {
        for (l, r) in iter.tuples().take(WAV_BLOCK_SIZE) {
            let l = l?;
            let r = r?;

            res.push(vec2(l, r));
        }
    } else {
        panic!("Unsupported channel count")
    }

    Ok(())
}

impl WavDecodeStream {
    fn read_next_block(&mut self) -> Result<&[Frame]> {
        self.current_block.clear();
        // Convert samples as appropriate
        match (self.format, self.bits_per_sample) {
            (SampleFormat::Int, 16) => append_frames(
                self.streamer.samples::<i16>().map_ok(|v| v.to_f32()),
                &mut self.current_block,
                self.channels,
            ),
            (SampleFormat::Int, 24) => append_frames(
                self.streamer
                    .samples::<i32>()
                    .map_ok(|v| ((v >> 8) as i16).to_f32()),
                &mut self.current_block,
                self.channels,
            ),
            (SampleFormat::Int, 32) => append_frames(
                self.streamer
                    .samples::<i32>()
                    .map_ok(|v| ((v >> 16) as i16).to_f32()),
                &mut self.current_block,
                self.channels,
            ),
            (SampleFormat::Float, 32) => append_frames(
                self.streamer.samples::<f32>().take(WAV_BLOCK_SIZE),
                &mut self.current_block,
                self.channels,
            ),
            _ => {
                panic!("Unsupported wav format")
            }
        }?;

        self.cursor = 0;

        Ok(&self.current_block)
    }
}

impl Source for WavDecodeStream {
    #[inline]
    fn next_sample(&mut self) -> Option<Frame> {
        if let Some(&val) = self.current_block.get(self.cursor) {
            self.cursor += 1;
            Some(val)
        } else {
            let &s = self.read_next_block().unwrap().get(1)?;
            self.cursor += 1;
            Some(s)
        }
    }

    fn sample_rate(&self) -> crate::SampleRate {
        self.sample_rate
    }

    fn sample_count(&self) -> Option<u64> {
        Some(self.decoded_len as _)
    }
}
