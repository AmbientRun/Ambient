wasmtime::component::bindgen!("main.bindings" in "wit");

pub mod shared {
    pub const INTERFACE_VERSION: u32 = include!("../../wit/INTERFACE_VERSION");
}
