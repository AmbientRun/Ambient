extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;

mod api_project;
mod main_macro;

const MANIFEST: &str = include_str!("../ambient.toml");

/// Makes your `main()` function accessible to the WASM host, and generates `components` and `concept` modules for your project.
///
/// If you do not add this attribute to your `main()` function, your module will not run.
#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    main_macro::main_impl(
        item.into(),
        api_project::read_file("ambient.toml".to_string()).expect("Failed to load ambient.toml"),
    )
    .unwrap()
    .into()
}

/// Generates global components and other boilerplate for the API crate.
#[proc_macro]
pub fn api_project(_input: TokenStream) -> TokenStream {
    TokenStream::from(
        api_project::implementation(
            (None, MANIFEST.to_string()),
            syn::Path::from(syn::Ident::new("crate", Span::call_site())),
            true,
            true,
        )
        .unwrap(),
    )
}
