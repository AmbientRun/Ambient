pub use host::*;

wit_bindgen_host_wasmtime_rust::export!(
    "../../../guest/rust/crates/elements_base_scripting_interface/src/internal/host.wit"
);
wit_bindgen_host_wasmtime_rust::import!(
    "../../../guest/rust/crates/elements_base_scripting_interface/src/internal/guest.wit"
);

pub mod shared {
    // extremely bad no good hack necessary because of https://github.com/bytecodealliance/wit-bindgen/issues/293
    include!(
        "../../../../../guest/rust/crates/elements_base_scripting_interface/src/internal/shared.rs"
    );
}
