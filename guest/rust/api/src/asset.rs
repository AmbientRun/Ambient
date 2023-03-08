#[cfg(feature = "server")]
use crate::internal::wit;

/// Resolves a asset path for an Ambient asset in this project to an absolute URL.
#[cfg(feature = "server")]
pub fn url(path: impl AsRef<str>) -> Option<String> {
    wit::server_asset::url(path.as_ref())
}
