mod event;
pub use event::*;

mod state;
pub use state::*;

mod runtime;
pub use runtime::*;

mod entity_id;
pub use entity_id::*;

// Re-exports from other crates.
pub use glam::{f32::*, u32::*, Vec2Swizzles, Vec3Swizzles, Vec4Swizzles};

#[inline]
/// Helper function that returns the [Default](std::default::Default::default) for the type `T`.
/// Most useful with struct update syntax, or with initializing components.
pub fn default<T: Default>() -> T {
    std::default::Default::default()
}

/// The version of this WASM interface. If this version is different to that of the running
/// host version, the module will panic and refuse to run.
#[doc(hidden)]
pub use crate::internal::wit::INTERFACE_VERSION;
