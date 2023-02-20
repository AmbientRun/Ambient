use std::{io, path::PathBuf};

use ambient_std::download_asset::AssetError;
use cpal::{BuildStreamError, PlayStreamError};
use lewton::VorbisError;
use thiserror::Error;

use crate::hrtf;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to find audio output device")]
    NoOutputDevice,
    #[error("Failed to find appropriate audio config")]
    NoOutputConfig,
    #[error("Default stream config error")]
    DefaultStreamConfigError(#[from] cpal::DefaultStreamConfigError),
    #[error("Failed to build output stream")]
    BuildStreamError(#[from] BuildStreamError),
    #[error("Failed to play output stream")]
    PlayStreamError(#[from] PlayStreamError),
    #[error("Failed to decode wav")]
    WavError(#[from] hound::Error),
    #[error("Unsupported file format: {0:?}")]
    UnsupportedFormat(String),
    #[error("Failed to open {1:?}: {0}")]
    Io(io::Error, PathBuf),
    #[error("Failed to download audio content")]
    ContentDownload(#[from] AssetError),
    #[error("Failed to decode vorbis")]
    Vorbis(#[from] VorbisError),
    #[error("Invalid channel configuration: {0}")]
    InvalidChannelCount(u16),
    #[error("Failed to load IR sphere for spatial audio")]
    IrSphere(hrtf::IrSphereError),

    #[error("Too many channels in ogg stream. Expected a maximum of 2 channels, found {0}")]
    TooManyOggChannels(usize),
}

pub(crate) type Result<T> = std::result::Result<T, Error>;
