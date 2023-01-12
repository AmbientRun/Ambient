//! # The Elements Scripting Interface
//! Welcome to the Elements scripting interface! Elements is a platform for making your own games
//! collaboratively, and scripting is a big part of that. Pleased to have you onboard!
//!
//! # Getting started
//! If you interact with Rules mode, you're already using scripts. From here, hit the
//! `Open scripts` button to open up the workspace for your game in Visual Studio Code.
//!
//! From there, you should be able to find the default scripts. Feel free to modify
//! these scripts; it'll update as soon as you save. Your code editor should be offering you
//! autocompletion and hints for every function exposed by this interface, but this docs page
//! is available if you need a reference!

#![deny(missing_docs)]

#[allow(missing_docs)]
mod host {
    wit_bindgen_guest_rust::import!("src/internal/host.wit");
    pub use self::host::*;
}

/// Entity-related functionality, including manipulation, creation, removal, and search.
pub mod entity;
/// All core events
pub mod events;
/// Physics-related functionality, including applying forces, changing physical properties, and more.
pub mod physics;

mod global;
pub use global::*;

pub(crate) mod internal;

pub use internal::{
    async_helpers::*,
    component::{
        change_query, despawn_query, internal_get_component, query, spawn_query, ChangeQuery, Component, Components, ComponentsTuple,
        EventQuery, GeneralQuery, GeneralQueryBuilder, LazyComponent, QueryEvent, SupportedComponentTypeGet, SupportedComponentTypeSet,
    },
    runtime::{frametime, on, on_async, once, once_async, run_async, time, EventOk, EventResult},
    shared::INTERFACE_VERSION,
    EntityId, EntityUid, ObjectRef,
};

pub use anyhow::{anyhow, Context as AnyhowContext};
pub use glam::{self, f32::*, Vec2Swizzles, Vec3Swizzles, Vec4Swizzles};
pub use once_cell;
pub use rand::prelude::*;

#[inline]
/// Helper function that returns the [std::default::Default::default] for the type `T`.
/// Most useful with struct update syntax, or with initializing components.
pub fn default<T: Default>() -> T {
    std::default::Default::default()
}
