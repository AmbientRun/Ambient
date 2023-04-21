use thiserror::Error;

use crate::internal::wit;

#[derive(Error, Debug)]
/// Errors that can occur when resolving an asset URL.
pub enum UrlError {
    #[error("Invalid URL: {0}")]
    /// The URL is invalid.
    InvalidUrl(String),
}
impl From<wit::asset::UrlError> for UrlError {
    fn from(value: wit::asset::UrlError) -> Self {
        match value {
            wit::asset::UrlError::InvalidUrl(err) => UrlError::InvalidUrl(err),
        }
    }
}

/// Resolves a asset path for an Ambient asset in this project to an absolute URL.
pub fn url(path: impl AsRef<str>) -> Result<String, UrlError> {
    Ok(wit::asset::url(path.as_ref())?)
}
