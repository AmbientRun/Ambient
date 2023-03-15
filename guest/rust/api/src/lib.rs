//! # The Ambient Rust API
//!
//! Welcome to the Ambient Rust API! This API allows you to write logic for Ambient, the multiplayer game engine, in Rust.
//!
//! The Ambient Book can be found [here](https://ambientrun.github.io/Ambient/).
//!
//! Ambient has first-class support for Rust. Please report any issues you encounter to the repository.
#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

/// Asset-related functionality, including retrieval of assets and where to find them.
pub mod asset;
/// ECS-related functionality not directly related to entities.
pub mod ecs;
/// Entity-related functionality, including manipulation, creation, removal, and search.
pub mod entity;
/// Event-related functionality, including sending events and standard events.
pub mod event;
/// Global functions and types for your convenience.
pub mod global;
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
pub use glam;
pub use once_cell;
pub use rand;

// Hi there! This macro generates the components that are exposed to you as a Ambient API user.
// We suggest that you look at the docs for this crate.
// Your IDE should also tell you about the components present here and show their corresponding
// doc comments.
ambient_api_macros::api_project!();
