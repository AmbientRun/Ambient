/// In Rust, functions that can fail are expected to return a [Result] type.
/// [EventResult] is a [Result] type that has no value and automatically
/// captures errors for you, which is why it's used as the return type
/// event handlers.
///
/// This accepts any kind of error,
/// so you can use the question-mark operator `?` to bubble errors up.
pub type EventResult = anyhow::Result<()>;

/// The default "happy path" value for an [EventResult]. You can return this
/// from an event handler to signal that everything's OK.
#[allow(non_upper_case_globals)]
pub const EventOk: EventResult = Ok(());
