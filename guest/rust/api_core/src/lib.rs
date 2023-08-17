//! # The Ambient Rust API
//!
//! Welcome to the Ambient Rust API! This API allows you to write logic for Ambient, the multiplayer game engine, in Rust.
//!
//! The Ambient Book can be found [here](https://ambientrun.github.io/Ambient/).
//!
//! Ambient has first-class support for Rust. Please report any issues you encounter to the repository.
#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(feature = "client")]
#[doc(hidden)]
pub mod client;
#[cfg(feature = "server")]
#[doc(hidden)]
pub mod server;

/// Retrieval of assets and where to find them.
pub mod asset;
/// ECS-related functionality not directly related to entities.
pub mod ecs;
/// Manipulation, creation, removal, search and more for entities.
pub mod entity;
/// Global functions and types for your convenience.
pub mod global;
/// Messaging to other modules and to the other side of the network boundary.
pub mod message;
/// Player-specific functionality.
pub mod player;

/// Helpful imports that almost all Ambient embers will use.
pub mod prelude;

/// Animation functions
pub mod animation;

/// Internal implementation details.
mod internal;

pub use ambient_api_macros::main;

use internal::generated::ambient_core;
pub use internal::generated::ambient_core as core;

#[allow(clippy::single_component_path_imports)]
use once_cell;
