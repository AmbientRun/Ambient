use std::{
    borrow::Cow,
    fmt::{Display, Formatter},
};

#[cfg(feature = "uncategorized")]
mod uncategorized;
use semver::Version;
#[cfg(feature = "uncategorized")]
pub use uncategorized::*;

pub mod events;
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, PartialOrd, Eq, Ord)]
pub struct AmbientVersion {
    pub version: Version,
    pub revision: String,
}

impl AmbientVersion {
    /// Returns git tag for this version, if any. Currently only available for releases and nightly builds.
    pub fn tag(&self) -> Option<String> {
        if !self.version.build.is_empty() {
            return None;
        }
        (self.version.pre.is_empty() || self.version.pre.starts_with("nightly-"))
            .then(|| format!("v{}", self.version))
    }
}

impl Default for AmbientVersion {
    fn default() -> Self {
        Self {
            version: Version::parse(RUNTIME_VERSION).expect("Failed to parse version"),
            revision: ambient_git_rev::REV_FULL.to_string(),
        }
    }
}

/// The version of the Ambient runtime. Where full specificity is required,
/// consider using [ambient_version] instead.
pub const RUNTIME_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The user agent to use when making requests as the runtime.
/// This is kept here to maintain one source of truth for the version.
pub const RUNTIME_USER_AGENT: &str = concat!("Ambient/", env!("CARGO_PKG_VERSION"));

impl Display for AmbientVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.version)?;
        if !self.revision.is_empty() {
            write!(f, "-{}", self.revision)?;
        }

        Ok(())
    }
}

pub fn ambient_version() -> AmbientVersion {
    AmbientVersion::default()
}

#[test]
fn test_ambient_version_tag() {
    assert_eq!(
        AmbientVersion {
            version: Version::parse("0.2.0").unwrap(),
            revision: "".to_string()
        }
        .tag(),
        Some("v0.2.0".to_string())
    );
    assert_eq!(
        AmbientVersion {
            version: Version::parse("0.3.0-dev").unwrap(),
            revision: "".to_string()
        }
        .tag(),
        None
    );
    assert_eq!(
        AmbientVersion {
            version: Version::parse("0.3.0+meta").unwrap(),
            revision: "".to_string()
        }
        .tag(),
        None
    );
    assert_eq!(
        AmbientVersion {
            version: Version::parse("0.3.0-nightly-2023-09-07").unwrap(),
            revision: "".to_string()
        }
        .tag(),
        Some("v0.3.0-nightly-2023-09-07".to_string())
    );
}
