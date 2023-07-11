extern crate proc_macro;

use ambient_project_macro_common::ManifestSource;
use proc_macro::TokenStream;

/// Generates global components and other boilerplate.
#[proc_macro]
pub fn host_project(_input: TokenStream) -> TokenStream {
    TokenStream::from(
        ambient_project_macro_common::generate_code(
            vec![(ManifestSource::Array(ambient_schema::FILES), false, true)],
            ambient_project_macro_common::Context::Host,
        )
        .unwrap(),
    )
}
