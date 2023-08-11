wasm_bridge::component::bindgen!({
    path: "wit",
    async: true,
});

pub use ambient::bindings::*;
pub use exports::ambient::bindings::guest;
