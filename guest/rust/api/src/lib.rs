//! # The Ambient Rust API
//!
//! Welcome to the Ambient Rust API! This API allows you to write logic for Ambient, the multiplayer game engine, in Rust.
//!
//! The Ambient Book can be found [here](https://ambientrun.github.io/Ambient/).
//!
//! Ambient has first-class support for Rust. Please report any issues you encounter to the repository.
#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]


/// Audio functions, loading sound and playback.
pub mod audio;
/// Asset-related functionality, including retrieval of assets and where to find them.
pub mod asset;
/// ECS-related functionality not directly related to entities.
pub mod ecs;
/// Entity-related functionality, including manipulation, creation, removal, and search.
pub mod entity;
/// Global functions and types for your convenience.
pub mod global;
/// Messaging to other modules and to the other side of the networking.
pub mod message;
/// Player-related functionality.
pub mod player;

/// Physics-related functionality, including applying forces, changing physical properties, and more.
#[cfg(feature = "server")]
pub mod physics;

/// Helpful imports that almost all Ambient projects will use.
pub mod prelude;

/// Internal implementation details.
mod internal;

pub use ambient_api_macros::main;

/// Re-exports from other crates.
pub use anyhow;
pub use futures;
pub use glam;
pub use once_cell;
pub use rand;

pub use internal::generated::*;
