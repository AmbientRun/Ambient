extern crate proc_macro;

use std::path::Path;

use ambient_package_macro_common::RetrievableFile;
use proc_macro::TokenStream;

mod main_macro;

/// Makes your `main()` function accessible to the WASM host, and generates a
/// `packages` module that contain all packages visible to your module.
///
/// If you do not add this attribute to your `main()` function, your module will not run.
#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    main_macro::main(
        item.clone().into(),
        RetrievableFile::Path(
            Path::new(&std::env::var("CARGO_MANIFEST_DIR").expect("no manifest dir"))
                .join("ambient.toml"),
        ),
    )
    .into()
}
