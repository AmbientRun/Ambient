pub mod client;
pub mod server;
pub mod shared;

pub(crate) static WASMTIME_ENGINE: once_cell::sync::Lazy<wasmtime::Engine> =
    once_cell::sync::Lazy::new(|| {
        let mut config = wasmtime::Config::new();
        config.wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable);
        config.wasm_component_model(true);
        wasmtime::Engine::new(&config).unwrap()
    });
