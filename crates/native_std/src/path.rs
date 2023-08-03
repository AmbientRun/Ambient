// This code was adapted from cargo-util (MIT/Apache2):
//
// https://github.com/rust-lang/cargo/blob/74c7aab19a9b3a045e7af13319409a9de2cf4ef7/crates/cargo-util/src/paths.rs#L74

use std::path::{Path, PathBuf};

/// Normalize a path, removing things like `.` and `..`.
///
/// CAUTION: This does not resolve symlinks (unlike
/// [`std::fs::canonicalize`]). This may cause incorrect or surprising
/// behavior at times. This should be used carefully. Unfortunately,
/// [`std::fs::canonicalize`] can be hard to use correctly, since it can often
/// fail, or on Windows returns annoying device paths. This is a problem Cargo
/// needs to improve on.
pub fn normalize(path: &Path) -> PathBuf {
    use std::path::Component;

    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}

/// Convert a path to a string using `/` as a separator independent of the platform.
#[cfg(feature = "uncategorized")]
pub fn path_to_unix_string(path: impl AsRef<Path>) -> String {
    use std::borrow::Cow;
    use std::path::Component;

    itertools::Itertools::intersperse(
        path.as_ref().components().map(|c| match c {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => Cow::Borrowed(""),
            Component::CurDir => Cow::Borrowed("."),
            Component::ParentDir => Cow::Borrowed(".."),
            Component::Normal(c) => c.to_string_lossy(),
        }),
        Cow::Borrowed("/"),
    )
    .collect()
}
