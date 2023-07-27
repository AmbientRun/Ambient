use std::borrow::Cow;

#[cfg(feature = "uncategorized")]
mod uncategorized;
#[cfg(feature = "uncategorized")]
pub use uncategorized::*;

pub mod events;
pub mod line_uid;
pub use ambient_cb::*;

/// Generate a new Ulid based on the current time regardless of platform
///
/// Uses `Data.now` on the web
pub fn ulid() -> ulid::Ulid {
    #[allow(clippy::disallowed_types)]
    // Retrieve a "normal" std::time::SystemTime regardless of platform
    let now = std::time::SystemTime::UNIX_EPOCH
        .checked_add(
            ambient_sys::time::SystemTime::now()
                .duration_since(ambient_sys::time::SystemTime::UNIX_EPOCH)
                .expect("Current time is before UNIX_EPOCH"),
        )
        .expect("Current system time could not be represented");

    ulid::Ulid::from_datetime(now)
}

/// Read a file as a string during debug at runtime, or use include_str at release
/// # Panics
/// Panics if the file can not be read (debug_assertions only)
#[macro_export]
macro_rules! include_file {
    ($f:expr) => {{
        #[cfg(feature = "hotload-includes")]
        {
            let mut path = std::path::PathBuf::from(file!());
            path.pop();
            path.push($f);
            let content =
                std::fs::read_to_string(&path).expect(&format!("Failed to read file {:?}", path));
            content
        }
        #[cfg(not(feature = "hotload-includes"))]
        {
            let content = include_str!($f);
            content.to_string()
        }
    }};
}

/// Read a file as a byte vec during debug at runtime, or use include_bytes at release
/// # Panics
/// Panics if the file can not be read (debug_assertions only)
#[macro_export]
macro_rules! include_file_bytes {
    ($f:expr) => {{
        #[cfg(feature = "hotload-includes")]
        {
            let mut path = std::path::PathBuf::from(file!());
            path.pop();
            path.push($f);
            let content = std::fs::read(&path).expect(&format!("Failed to read file {:?}", path));
            content
        }
        #[cfg(not(feature = "hotload-includes"))]
        {
            let content = include_bytes!($f);
            content.to_vec()
        }
    }};
}

pub fn log_error(err: &anyhow::Error) {
    #[cfg(feature = "sentry")]
    sentry_anyhow::capture_anyhow(err);
    #[cfg(not(feature = "sentry"))]
    tracing::error!("{:?}", err);
}

#[macro_export]
/// Consumes and logs the error variant.
///
/// The Ok variant is discarded.
macro_rules! log_result {
    ( $x:expr ) => {
        if let Err(err) = $x {
            $crate::log_error(&err.into());
        }
    };
}

#[macro_export]
macro_rules! log_warning {
    ( $x:expr ) => {
        if let Err(err) = $x {
            log::warn!("{:?}", err);
        }
    };
}

#[macro_export]
macro_rules! unwrap_log_err {
    ( $x:expr ) => {
        match $x {
            Ok(val) => val,
            Err(err) => {
                $crate::log_error(&err.into());
                return Default::default();
            }
        }
    };
}

#[macro_export]
macro_rules! unwrap_log_warn {
    ( $x:expr ) => {
        match $x {
            Ok(val) => val,
            Err(err) => {
                log::warn!("{:?}", err);
                return Default::default();
            }
        }
    };
}

pub type CowStr = Cow<'static, str>;

pub fn to_byte_unit(bytes: u64) -> String {
    if bytes < 1024 * 10 {
        format!("{bytes} b")
    } else if bytes < 1024 * 1024 * 10 {
        format!("{} kb", bytes / 1024)
    } else if bytes < 1024 * 1024 * 1024 * 10 {
        format!("{} mb", bytes / 1024 / 1024)
    } else {
        format!("{} gb", bytes / 1024 / 1024 / 1024)
    }
}

pub fn parse_git_revision(version: &str) -> Option<String> {
    let s = version.split('-').collect::<Vec<_>>();
    if s.len() <= 2 {
        Some(s[0].to_string())
    } else {
        Some(s.get(3)?[1..].to_string())
    }
}

pub fn git_revision() -> Option<String> {
    parse_git_revision(git_version::git_version!(
        args = ["--abbrev=10", "--always", "--long"]
    ))
}

#[test]
fn test_parse_git_revision() {
    assert_eq!(parse_git_revision("9f244c3"), Some("9f244c3".to_string()));
    assert_eq!(
        parse_git_revision("9f244c3-modified"),
        Some("9f244c3".to_string())
    );
    assert_eq!(
        parse_git_revision("git-0.3.0-dev-g9f244c3"),
        Some("9f244c3".to_string())
    );
}

#[test]
fn test_git_revision() {
    let revision = String::from_utf8(
        std::process::Command::new("git")
            .args(["rev-parse", "--short=10", "HEAD"])
            .output()
            .expect("Failed to init git repo")
            .stdout,
    )
    .expect("Invalid encoding");
    assert_eq!(git_revision().as_deref(), Some(revision.trim()));
}
