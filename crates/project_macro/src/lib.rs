extern crate proc_macro;

use proc_macro::TokenStream;

/// Generates global components and other boilerplate.
#[proc_macro]
pub fn host_project(_input: TokenStream) -> TokenStream {
    let ts = TokenStream::from(
        ambient_project_macro_common::implementation(
            (None, ambient_project_macro_common::MANIFEST.to_string()),
            ambient_project_macro_common::Context::Host,
            true,
            true,
        )
        .unwrap(),
    );
    std::fs::write("/Users/mithunhunsur/Documents/Work/Tilt/output.txt", ts.to_string()).unwrap();
    ts
}
