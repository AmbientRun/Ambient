extern crate proc_macro;

use proc_macro::TokenStream;
use std::path::Path;

/// Generates global components and other boilerplate.
#[proc_macro]
pub fn host_project(_input: TokenStream) -> TokenStream {
    TokenStream::from(
        ambient_project_macro_common::implementation(
            Path::new(ambient_schema::MANIFEST_PATH),
            ambient_project_macro_common::Context::Host,
            true,
            true,
        )
        .unwrap(),
    )
}
