use std::sync::Arc;

use derive_more::From;

use crate::{
    error::Result,
    vorbis::{VorbisDecodeStream, VorbisTrack},
    wav::{WavDecodeStream, WavTrack},
    Source,
};

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum AudioFormat {
    Wav,
    Vorbis,
}

#[derive(From, Debug, Clone)]
/// Represents a buffer-backed audio source, like a WAV file.
///
/// Cloning the track does not duplicate the buffer memory.
pub enum Track {
    Vorbis(VorbisTrack),
    Wav(WavTrack),
}

impl Track {
    pub fn from_wav(bytes: impl Into<Arc<[u8]>>) -> Result<Self> {
        Ok(Self::Wav(WavTrack::new(bytes.into())?))
    }

    pub fn from_vorbis(bytes: impl Into<Arc<[u8]>>) -> Result<Self> {
        Ok(Self::Vorbis(VorbisTrack::new(bytes.into())?))
    }

    pub fn from_format(bytes: impl Into<Arc<[u8]>>, format: AudioFormat) -> Result<Self> {
        match format {
            AudioFormat::Wav => Self::from_wav(bytes.into()),
            AudioFormat::Vorbis => Self::from_vorbis(bytes.into()),
        }
    }

    pub fn decode(&self) -> TrackDecodeStream {
        match self {
            Track::Vorbis(v) => TrackDecodeStream::Vorbis(Box::new(v.decode())),
            Track::Wav(v) => TrackDecodeStream::Wav(v.decode()),
        }
    }
}

#[derive(Clone)]
pub enum TrackDecodeStream {
    Vorbis(Box<VorbisDecodeStream>),
    Wav(WavDecodeStream),
}

impl Source for TrackDecodeStream {
    fn next_sample(&mut self) -> Option<crate::Frame> {
        match self {
            TrackDecodeStream::Vorbis(v) => v.next_sample(),
            TrackDecodeStream::Wav(v) => v.next_sample(),
        }
    }

    fn sample_buffered(&mut self, output: &mut [crate::Frame]) -> usize {
        match self {
            TrackDecodeStream::Vorbis(v) => v.sample_buffered(output),
            TrackDecodeStream::Wav(v) => v.sample_buffered(output),
        }
    }

    fn sample_rate(&self) -> crate::SampleRate {
        match self {
            TrackDecodeStream::Vorbis(v) => v.sample_rate(),
            TrackDecodeStream::Wav(v) => v.sample_rate(),
        }
    }

    fn sample_count(&self) -> Option<u64> {
        match self {
            TrackDecodeStream::Vorbis(v) => v.sample_count(),
            TrackDecodeStream::Wav(v) => v.sample_count(),
        }
    }
}
