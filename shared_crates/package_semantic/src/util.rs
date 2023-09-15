use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum ReadFileError {
    #[error("file reading is not supported on web")]
    FileReadingNotSupportedOnWeb,
    #[error("failed to read path {file_path:?}")]
    FailedToReadPath { file_path: String },
    #[error("failed to get URL {url:?}")]
    FailedToGetUrl { url: Url },
    #[error("failed to get text from URL {url:?}")]
    FailedToGetTextFromUrl { url: Url },
}

pub async fn read_file(url: &Url) -> Result<String, ReadFileError> {
    if url.scheme() == "file" {
        #[cfg(target_os = "unknown")]
        return Err(ReadFileError::FileReadingNotSupportedOnWeb);

        #[cfg(not(target_os = "unknown"))]
        if let Ok(file_path) = url.to_file_path() {
            return std::fs::read_to_string(&file_path).map_err(|_| {
                ReadFileError::FailedToReadPath {
                    file_path: file_path.to_string_lossy().to_string(),
                }
            });
        }
    }

    reqwest::get(url.clone())
        .await
        .map_err(|_| ReadFileError::FailedToGetUrl { url: url.clone() })?
        .text()
        .await
        .map_err(|_| ReadFileError::FailedToGetTextFromUrl { url: url.clone() })
}
