mod state;
pub use state::*;

mod runtime;
pub use runtime::*;

mod entity_id;
pub use entity_id::*;

mod shapes;
pub use shapes::*;

/// Spatial audio system
pub mod spatial_audio;

// Re-exports from other crates.
pub use ambient_shared_types::{CursorIcon, ModifiersState, MouseButton, VirtualKeyCode};
pub use futures::{Future, FutureExt};
pub use glam::{f32::*, u32::*, Vec2Swizzles, Vec3Swizzles, Vec4Swizzles};
pub use ulid::Ulid;

/// In Rust, functions that can fail are expected to return a [Result] type.
/// [ResultEmpty] is a [Result] type that has no value and can accept
/// any kind of error through the question-mark operator `?`.
///
/// It is used as the default return type for Ambient operations that take
/// a callback.
pub type ResultEmpty = anyhow::Result<()>;

/// The default "happy path" value for an [ResultEmpty]. You can return this
/// from a handler to signal that there are no issues.
#[allow(non_upper_case_globals)]
pub const OkEmpty: ResultEmpty = Ok(());

#[inline]
/// Helper function that returns the [Default](std::default::Default::default) for the type `T`.
/// Most useful with struct update syntax, or with initializing components.
pub fn default<T: Default>() -> T {
    std::default::Default::default()
}
