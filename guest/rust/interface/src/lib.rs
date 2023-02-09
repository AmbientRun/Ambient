//! # The Elements Scripting Interface
//! Welcome to the Elements scripting interface!
#![deny(missing_docs)]

#[allow(missing_docs)]
mod host {
    wit_bindgen_guest_rust::import!("wit/host.wit");
    pub use self::host::*;
}

/// Entity-related functionality, including manipulation, creation, removal, and search.
pub mod entity;
/// Event-related functionality, including sending events and standard events.
pub mod event;
/// Physics-related functionality, including applying forces, changing physical properties, and more.
pub mod physics;
/// Player-related functionality.
pub mod player;

/// Global functions and types for your convenience.
mod global;
pub use global::*;

/// Internal implementation details. The relevant details are exported.
mod internal;
pub use internal::component::{
    change_query, despawn_query, internal_get_component, query, spawn_query, ChangeQuery,
    Component, Components, ComponentsTuple, EventQuery, GeneralQuery, GeneralQueryBuilder,
    LazyComponent, QueryEvent, SupportedComponentTypeGet, SupportedComponentTypeSet,
};

pub use interface_macros::main;

/// Re-exports from other crates.
pub use anyhow::{anyhow, Context as AnyhowContext};
pub use glam::{self, f32::*, Vec2Swizzles, Vec3Swizzles, Vec4Swizzles};
pub use once_cell;
pub use rand::prelude::*;

/// The version of this scripting interface. If this version is different to that of the running
/// host version, the script will panic and refuse to run.
#[doc(hidden)]
pub const INTERFACE_VERSION: u32 = include!("../wit/INTERFACE_VERSION");

interface_macros::elements_project!(extend = []);

#[inline]
/// Helper function that returns the [std::default::Default::default] for the type `T`.
/// Most useful with struct update syntax, or with initializing components.
pub fn default<T: Default>() -> T {
    std::default::Default::default()
}
