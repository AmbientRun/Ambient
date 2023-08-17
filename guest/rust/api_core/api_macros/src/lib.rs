extern crate proc_macro;

use std::path::Path;

use ambient_ember_macro_common::ManifestSource;
use proc_macro::TokenStream;

mod main_macro;

/// Makes your `main()` function accessible to the WASM host, and generates a
/// `embers` module that contain all embers visible to your module.
///
/// If you do not add this attribute to your `main()` function, your module will not run.
#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    main_macro::main(
        item.clone().into(),
        ManifestSource::Path {
            ember_path: Path::new(&std::env::var("CARGO_MANIFEST_DIR").expect("no manifest dir")),
        },
    )
    .into()
}
