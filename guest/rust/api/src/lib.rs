//! # The Ambient Rust API
//!
//! Welcome to the Ambient Rust API! This API allows you to write logic for Ambient, the multiplayer game engine, in Rust.
//!
//! The Ambient Book can be found [here](https://ambientrun.github.io/Ambient/).
//!
//! Ambient has first-class support for Rust. Please report any issues you encounter to the repository.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(feature = "client")]
pub use ambient_api_core::client::*;
#[cfg(feature = "server")]
pub use ambient_api_core::server::*;
pub use ambient_api_core::*;

pub use ambient_ui;

pub mod prelude {
    pub use ambient_api_core::prelude::*;
    pub use ambient_ui::prelude::*;
}

/// Re-exports from other crates.
pub use anyhow;
pub use futures;
pub use glam;
pub use once_cell;
pub use rand;
