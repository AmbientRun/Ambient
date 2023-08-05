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
#[doc(hidden)]
pub fn url(ember_id: &str, path: &str) -> Result<String, UrlError> {
    Ok(wit::asset::url(ember_id, path)?)
}

#[cfg(feature = "server")]
/// On the server, attempts to rebuild all compatible WASM modules in the project.
///
/// NOTE: This may be removed at a later stage. It primarily exists to enable WASM rebuilding.
pub async fn build_wasm() -> anyhow::Result<()> {
    use crate::{core::messages::WasmRebuild, global};
    wit::server_asset::build_wasm();
    match global::wait_for_runtime_message::<WasmRebuild>(|_| true)
        .await
        .error
    {
        Some(err) => anyhow::bail!("{err}"),
        None => Ok(()),
    }
}
