extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

mod elements_project;

/// Makes your main() function accessible to the scripting host.
///
/// If you do not add this attribute to your main() function, your script will not run.
#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as syn::ItemFn);
    let fn_name = item.sig.ident.clone();
    if item.sig.asyncness.is_none() {
        panic!("the `{fn_name}` function must be async");
    }

    quote! {
        #item

        #[no_mangle]
        pub extern "C" fn call_main(runtime_interface_version: u32) {
            if INTERFACE_VERSION != runtime_interface_version {
                panic!("This module was compiled with interface version {{INTERFACE_VERSION}}, but the host is running with version {{runtime_interface_version}}.");
            }
            run_async(#fn_name());
        }
    }.into()
}

/// Parses your project's manifest and generates components and other boilerplate.
#[proc_macro]
pub fn elements_project(input: TokenStream) -> TokenStream {
    let extend_paths: Option<Vec<Vec<String>>> = if input.is_empty() {
        None
    } else {
        syn::custom_keyword!(extend);

        struct Extend {
            elems: syn::punctuated::Punctuated<syn::Path, syn::token::Comma>,
        }
        impl syn::parse::Parse for Extend {
            fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
                let _extend_token = input.parse::<extend>()?;
                let _equal_token = input.parse::<syn::Token![=]>()?;

                let content;
                let _bracket_token = syn::bracketed!(content in input);
                let mut elems = syn::punctuated::Punctuated::new();

                while !content.is_empty() {
                    let first: syn::Path = content.parse()?;
                    elems.push_value(first);
                    if content.is_empty() {
                        break;
                    }
                    let punct = content.parse()?;
                    elems.push_punct(punct);
                }

                Ok(Self { elems })
            }
        }

        let extend = syn::parse_macro_input!(input as Extend);
        Some(
            extend
                .elems
                .into_iter()
                .map(|p| {
                    p.segments
                        .into_iter()
                        .map(|s| s.ident.to_string())
                        .collect()
                })
                .collect(),
        )
    };

    TokenStream::from(elements_project_pm2(extend_paths).unwrap())
}

fn elements_project_pm2(
    extend_paths: Option<Vec<Vec<String>>>,
) -> anyhow::Result<proc_macro2::TokenStream> {
    elements_project::implementation(
        elements_project::read_file("elements.toml".to_string()).unwrap(),
        extend_paths
            .as_ref()
            .map(|a| a.as_slice())
            .unwrap_or_default(),
        extend_paths.is_some(),
    )
}
