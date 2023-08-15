extern crate proc_macro;

use std::path::Path;

use ambient_project_macro_common::ManifestSource;
use proc_macro::TokenStream;

mod main_macro;

/// Makes your `main()` function accessible to the WASM host, and generates `components` and `concept` modules for your project.
///
/// If you do not add this attribute to your `main()` function, your module will not run.
#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ts = main_macro::main_impl(
        item.into(),
        ManifestSource::Path {
            ember_path: Path::new(&std::env::var("CARGO_MANIFEST_DIR").expect("no manifest dir")),
        },
    );

    match ts {
        Ok(ts) => ts.into(),
        Err(e) => {
            let msg = format!(
                "Error while running Ambient ember macro: {}{}",
                e.to_string(),
                e.source()
                    .map(|e| format!("\nCaused by: {e}"))
                    .unwrap_or_default()
            );
            quote::quote! {
                compile_error!(#msg);
                #[no_mangle]
                #[doc(hidden)]
                fn main() {}
            }
        }
        .into(),
    }
}
