pub mod control;
pub mod task;
pub mod time;
pub mod timer;

/// Sets a panic hook which prints panics to the browser dev-console
#[cfg(all(target_arch = "wasm32", feature = "console_error_panic_hook"))]
pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    {
        tracing::info!("Setting panic hook");
        console_error_panic_hook::set_once();
    }
}

#[cfg_attr(not(target_arch = "wasm32"), path = "./native/mod.rs")]
#[cfg_attr(target_arch = "wasm32", path = "./wasm/mod.rs")]
pub(crate) mod platform;
