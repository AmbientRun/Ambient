pub mod control;
mod missed_tick;
pub mod task;
pub mod time;
// pub mod timer;

pub use missed_tick::MissedTickBehavior;

/// Sets a panic hook which prints panics to the browser dev-console
pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(all(target_os = "unknown", feature = "console_error_panic_hook"))]
    {
        tracing::info!("Setting panic hook");
        console_error_panic_hook::set_once();
    }
}

#[cfg_attr(not(target_os = "unknown"), path = "./native/mod.rs")]
#[cfg_attr(target_os = "unknown", path = "./wasm/mod.rs")]
pub(crate) mod platform;

pub use platform::clipboard;
/// Platform agnostic file io.
///
/// **Note**: wasm file io always return Err, but do *not* panic.
pub use platform::fs;
