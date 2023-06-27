wasmtime::component::bindgen!({
    path: "wit",
});

pub use ambient::bindings::*;
pub use exports::ambient::bindings::guest;
