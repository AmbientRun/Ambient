#![feature(coerce_unsized)]
#![feature(unsize)]
use std::{
    borrow::Cow, marker::Unsize, ops::{CoerceUnsized, Deref, DerefMut}, sync::Arc, time::{Duration, SystemTime}
};

pub use elements_asset_cache as asset_cache;
pub mod asset_url;
pub mod barc;
pub mod color;
pub mod colorspace;
pub mod disk_cache;
pub mod download_asset;
pub mod events;
pub mod fps_counter;
pub mod line_hash;
pub mod math;
pub mod mesh;
pub mod ordered_glam;
pub mod path;
pub mod shapes;
pub mod sparse_vec;
pub mod time;

pub use time::{FromDuration, IntoDuration};

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

/// This is just wrapping an Box, and it only exists because Box<dyn Fn..> doesn't implement Debug, so
/// we're wrapping it with a CbBox to avoid having to handle that in all structs that implement Debug
pub struct CbBox<T: ?Sized>(pub Box<T>);
impl<T> CbBox<T> {
    pub fn new(cb: T) -> Self {
        Self(Box::new(cb))
    }
}

impl<T: ?Sized> std::fmt::Debug for CbBox<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("CbBox").finish()
    }
}

impl<T: ?Sized> Deref for CbBox<T> {
    type Target = <Arc<T> as Deref>::Target;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}
impl<T: ?Sized> DerefMut for CbBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}

pub type Callback<T, Ret = ()> = Cb<dyn Fn(T) -> Ret + Sync + Send>;

/// This is just wrapping an Arc, and it only exists because Arc<dyn Fn..> doesn't implement Debug, so
/// we're wrapping it with a Cb to avoid having to handle that in all structs that implement Debug
#[derive(Default)]
pub struct Cb<T: ?Sized>(pub Arc<T>);

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Cb<U>> for Cb<T> {}
impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<CbBox<U>> for CbBox<T> {}

impl<T> Cb<T> {
    #[inline]
    pub fn new(val: T) -> Self {
        Self(Arc::new(val))
    }
}
impl<T: ?Sized> std::fmt::Debug for Cb<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Cb").finish()
    }
}
impl<T: ?Sized> Clone for Cb<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T: ?Sized> Deref for Cb<T> {
    type Target = <Arc<T> as Deref>::Target;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

pub fn log_error(err: &anyhow::Error) {
    #[cfg(feature = "sentry")]
    sentry_anyhow::capture_anyhow(err);
    #[cfg(not(feature = "sentry"))]
    tracing::error!("{:?}", err);
}

pub type CallbackFn<T, U = ()> = Arc<dyn Fn(T) -> U + Sync + Send + 'static>;
pub type CallbackBox<T, U = ()> = Box<dyn Fn(T) -> U + Sync + Send + 'static>;

pub type CellFn<T, U = ()> = dyn Fn(&mut T) -> U + Send + Sync;
pub type CellFnOnce<T, U = ()> = dyn Fn(&mut T) -> U + Send + Sync;

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

pub fn sha256_digest(value: &str) -> String {
    let digest = ring::digest::digest(&ring::digest::SHA256, value.as_bytes());
    data_encoding::HEXLOWER.encode(digest.as_ref())
}

pub fn from_now(time: SystemTime) -> Option<String> {
    let duration = SystemTime::now().duration_since(time).ok()?;
    Some(format!("{} ago", pretty_duration(duration)))
}
pub fn pretty_duration(duration: Duration) -> String {
    let mut secs = duration.as_secs();
    if secs == 0 {
        return format!("{} ms", duration.as_millis());
    }

    let years = secs / (86400.0 * 365.2422) as u64;
    secs %= (86400.0 * 365.2422) as u64;

    let days = secs / 86400;
    secs %= 86400;

    let hours = secs / 3600;
    secs %= 3600;

    let minutes = secs / 60;
    secs %= 60;

    let mut res = Vec::new();

    if years > 0 {
        res.push(format!("{years} years"))
    }

    if days > 0 {
        res.push(format!("{days} days"))
    }

    if years == 0 && days == 0 {
        if hours > 0 {
            res.push(format!("{hours} hours"))
        }

        if minutes > 0 {
            res.push(format!("{minutes} minutes"))
        }

        if secs > 0 {
            res.push(format!("{secs} seconds"))
        }
    }

    res.join(" ")
}
