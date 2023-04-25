extern crate proc_macro;

use proc_macro::TokenStream;
use std::path::PathBuf;

/// Generates global components and other boilerplate.
#[proc_macro]
pub fn host_project(_input: TokenStream) -> TokenStream {
    TokenStream::from(
        ambient_project_macro_common::implementation(
            &PathBuf::from(ambient_project_macro_common::MANIFEST_PATH),
            ambient_project_macro_common::Context::Host,
            true,
            true,
        )
        .unwrap(),
    )
}
