extern crate proc_macro;

use anyhow::Context;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;

mod ambient_project;

const MANIFEST: &str = include_str!("../ambient.toml");

/// Makes your `main()` function accessible to the WASM host, and generates `components` and `concept` modules for your project.
///
/// If you do not add this attribute to your `main()` function, your module will not run.
#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut item = syn::parse_macro_input!(item as syn::ItemFn);
    let fn_name = quote::format_ident!("async_{}", item.sig.ident);
    item.sig.ident = fn_name.clone();
    if item.sig.asyncness.is_none() {
        panic!("the `{fn_name}` function must be async");
    }

    let spans = Span::call_site();
    let mut path = syn::Path::from(syn::Ident::new("ambient_api", spans));
    path.leading_colon = Some(syn::Token![::](spans));
    let project_boilerplate = ambient_project::implementation(
        ambient_project::read_file("ambient.toml".to_string())
            .context("Failed to load ambient.toml")
            .unwrap(),
        path.clone(),
        false,
        true,
    )
    .unwrap();

    quote! {
        #project_boilerplate

        #item

        #[no_mangle]
        #[doc(hidden)]
        pub fn main() {
            #path::global::run_async(#fn_name());
        }
    }
    .into()
}

/// Generates global components and other boilerplate for the API crate.
#[proc_macro]
pub fn api_project(_input: TokenStream) -> TokenStream {
    TokenStream::from(
        ambient_project::implementation(
            (None, MANIFEST.to_string()),
            syn::Path::from(syn::Ident::new("crate", Span::call_site())),
            true,
            true,
        )
        .unwrap(),
    )
}
