wasm_bridge::component::bindgen!({
    path: "wit",
    async: false,
});

pub use ambient::bindings::*;
pub use exports::ambient::bindings::guest;
