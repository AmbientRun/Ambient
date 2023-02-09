wit_bindgen_host_wasmtime_rust::export!("wit/host.wit");
wit_bindgen_host_wasmtime_rust::import!("wit/guest.wit");

pub mod shared {
    pub const INTERFACE_VERSION: u32 = include!("../../wit/INTERFACE_VERSION");
}
