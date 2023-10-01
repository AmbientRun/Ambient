use std::path::Path;

use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum RetrieveError {
    #[error("file reading is not supported on web")]
    FileReadingNotSupportedOnWeb,
    #[error("failed to read path {file_path:?}")]
    FailedToReadPath { file_path: String },
    #[error("failed to get URL {url:?}")]
    FailedToGetUrl { url: Url },
    #[error("failed to get text from URL {url:?}")]
    FailedToGetTextFromUrl { url: Url },
}

pub fn retrieve_file(path: &Path) -> Result<String, RetrieveError> {
    #[cfg(target_os = "unknown")]
    return Err(RetrieveError::FileReadingNotSupportedOnWeb);

    #[cfg(not(target_os = "unknown"))]
    return std::fs::read_to_string(&path).map_err(|_| RetrieveError::FailedToReadPath {
        file_path: path.to_string_lossy().to_string(),
    });
}

pub async fn retrieve_url(url: &Url) -> Result<String, RetrieveError> {
    if url.scheme() == "file" {
        return retrieve_file(&url.to_file_path().ok().unwrap_or_default());
    }

    reqwest::get(url.clone())
        .await
        .map_err(|_| RetrieveError::FailedToGetUrl { url: url.clone() })?
        .text()
        .await
        .map_err(|_| RetrieveError::FailedToGetTextFromUrl { url: url.clone() })
}
