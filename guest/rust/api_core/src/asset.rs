use crate::internal::wit;

/// Resolves a asset path for an Ambient asset in this project to an absolute URL.
pub fn url(path: impl AsRef<str>) -> Option<String> {
    wit::asset::url(path.as_ref())
}
