extern crate proc_macro;

use ambient_project_macro_common::ManifestSource;
use proc_macro::TokenStream;
use std::path::Path;

/// Generates global components and other boilerplate.
#[proc_macro]
pub fn host_project(_input: TokenStream) -> TokenStream {
    TokenStream::from(
        ambient_project_macro_common::generate_code(
            ManifestSource::Path(Path::new(ambient_schema::MANIFEST_PATH).to_owned()),
            ambient_project_macro_common::Context::Host,
            true,
            true,
        )
        .unwrap(),
    )
}
