#[cfg(all(feature = "guest", not(feature = "native")))]
mod guest;
#[cfg(all(feature = "guest", not(feature = "native")))]
pub use guest::*;

#[cfg(feature = "native")]
mod native;
#[cfg(feature = "native")]
pub use native::*;
