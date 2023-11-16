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
                if ret.components().any(|c| matches!(c, Component::Normal(_))) {
                    ret.pop();
                } else {
                    ret.push(component.as_os_str());
                }
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}

/// Convert a path to a string using `/` as a separator independent of the platform, using lossy conversion if necessary.
pub fn path_to_unix_string_lossy(path: impl AsRef<Path>) -> String {
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

/// Convert a path to a string using `/` as a separator independent of the platform, returning `None` if any of the
/// components are not valid Unicode.
pub fn path_to_unix_string(path: impl AsRef<Path>) -> Option<String> {
    use std::borrow::Cow;
    use std::path::Component;

    itertools::Itertools::intersperse(
        path.as_ref().components().map(|c| match c {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => Some(Cow::Borrowed("")),
            Component::CurDir => Some(Cow::Borrowed(".")),
            Component::ParentDir => Some(Cow::Borrowed("..")),
            Component::Normal(c) => c.to_str().map(Cow::Borrowed),
        }),
        Some(Cow::Borrowed("/")),
    )
    .collect()
}

#[cfg(test)]
mod tests {
    use super::normalize;
    use std::path::Path;

    #[test]
    fn test_relative_path_simplification() {
        assert_eq!(
            normalize(Path::new("a/b/./../c")),
            Path::new("a/c").to_path_buf()
        );
        assert_eq!(
            normalize(Path::new("../../a/b/c/../d")),
            Path::new("../../a/b/d").to_path_buf()
        );
    }

    #[test]
    fn test_absolute_path_handling() {
        assert_eq!(
            normalize(Path::new("/a/b/../c")),
            Path::new("/a/c").to_path_buf()
        );
    }

    #[test]
    fn test_edge_cases() {
        assert_eq!(normalize(Path::new("..")), Path::new("..").to_path_buf());
        assert_eq!(normalize(Path::new(".")), Path::new("").to_path_buf());
    }

    #[test]
    fn test_mixed_components() {
        assert_eq!(
            normalize(Path::new("a/./b/.././c/d/..")),
            Path::new("a/c").to_path_buf()
        );
    }

    #[test]
    fn test_empty_and_single_component_paths() {
        assert_eq!(normalize(Path::new("")), Path::new("").to_path_buf());
        assert_eq!(normalize(Path::new("a")), Path::new("a").to_path_buf());
    }
}
