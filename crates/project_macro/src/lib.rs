extern crate proc_macro;

use proc_macro::TokenStream;

/// Generates global components and other boilerplate.
#[proc_macro]
pub fn host_project(_input: TokenStream) -> TokenStream {
    TokenStream::from(
        ambient_project_macro_common::implementation(
            (None, ambient_project_macro_common::MANIFEST.to_string()),
            ambient_project_macro_common::Context::Host,
            true,
            true,
        )
        .unwrap(),
    )
}
