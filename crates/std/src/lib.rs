#![cfg_attr(feature = "cb", feature(coerce_unsized))]
#![cfg_attr(feature = "cb", feature(unsize))]
use std::borrow::Cow;

#[cfg(feature = "uncategorized")]
mod uncategorized;
#[cfg(feature = "uncategorized")]
pub use uncategorized::*;

#[cfg(feature = "cb")]
pub mod cb;
#[cfg(feature = "cb")]
pub use cb::*;

pub mod colorspace;
pub mod events;
pub mod line_hash;
pub mod path;

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
            let content = std::fs::read_to_string(&path).expect(&format!("Failed to read file {:?}", path));
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

pub fn to_byte_unit(bytes: usize) -> String {
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
