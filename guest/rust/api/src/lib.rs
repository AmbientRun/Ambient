//! # The Ambient Rust API
//!
//! Welcome to the Ambient Rust API! This API allows you to write logic for the Ambient Runtime in Rust.
//!
//! You can find the Ambient Book at: <https://ambientorg.github.io/Ambient/>
//!
//! Ambient has first-class support for Rust. Please report any issues you encounter to the repository.
#![deny(missing_docs)]

/// ECS-related functionality not directly related to entities.
pub mod ecs;
/// Entity-related functionality, including manipulation, creation, removal, and search.
pub mod entity;
/// Event-related functionality, including sending events and standard events.
pub mod event;
/// Global functions and types for your convenience.
pub mod global;
/// Physics-related functionality, including applying forces, changing physical properties, and more.
pub mod physics;
/// Player-related functionality.
pub mod player;

/// Helpful imports that almost all Ambient projects will use.
pub mod prelude;

/// Internal implementation details.
mod internal;

pub use ambient_api_macros::main;

/// Re-exports from other crates.
pub use anyhow;
pub use glam;
pub use once_cell;

// Hi there! This macro generates the components that are exposed to you as a Ambient API user.
// We suggest that you look at the docs for this crate.
// Your IDE should also tell you about the components present here and show their corresponding
// doc comments.
ambient_api_macros::api_project!();
