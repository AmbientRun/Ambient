extern crate proc_macro;

use std::path::Path;

use ambient_package_macro_common::RetrievableFile;
use proc_macro::TokenStream;

mod main_macro;

/// Makes your `main()` function accessible to the WASM host, and generates a
/// `packages` module that contain all packages visible to your package, including itself.
///
/// If you do not add this attribute to your `main()` function, your module will not run.
#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ambient_toml = RetrievableFile::Path(
        Path::new(&std::env::var("CARGO_MANIFEST_DIR").expect("no manifest dir"))
            .join("ambient.toml"),
    );
    match main_macro::derive_main(item.clone().into(), ambient_toml) {
        Ok(v) => v.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
