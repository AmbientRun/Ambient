use std::sync::Arc;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("The sound effect {0:?} does not exist")]
    MissingEffect(String),
    #[error("There are no more available sinks")]
    NoAvailableSink,
    #[error(transparent)]
    AudioError(#[from] Arc<ambient_audio::Error>),
}
