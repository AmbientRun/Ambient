//! # The Tilt Scripting Interface
//! Welcome to the Tilt scripting interface!

#![deny(missing_docs)]

#[allow(missing_docs)]
mod host {
    wit_bindgen_guest_rust::import!("src/internal/host.wit");
    pub use self::host::*;
}

/// Player-related functionality.
pub mod player;

pub(crate) mod internal;

pub use tilt_base_scripting_interface::*;
