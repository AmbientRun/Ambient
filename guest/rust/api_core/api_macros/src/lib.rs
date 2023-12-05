extern crate proc_macro;

use proc_macro2::{Ident, Span, TokenTree};
use quote::{quote, ToTokens};

/// Makes your `main()` function accessible to the WASM host, and generates a
/// `packages` module that contain all packages visible to your package, including itself.
///
/// If you do not add this attribute to your `main()` function, your module will not run.
#[proc_macro_attribute]
pub fn main(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    main_impl(item.clone().into()).into()
}

fn main_impl(item: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let (item, parsed) = parse_function(item);

    let spans = Span::call_site();
    let mut path = syn::Path::from(syn::Ident::new("ambient_api", spans));
    path.leading_colon = Some(syn::Token![::](spans));

    let call_stmt = if let Some(ParsedFunction { fn_name, is_async }) = parsed {
        let call_expr = if is_async {
            quote! { #fn_name() }
        } else {
            quote! { async { #fn_name() } }
        };

        quote! { #path::global::run_async(#call_expr) }
    } else {
        quote! {}
    };

    quote! {
        #item

        #[no_mangle]
        #[doc(hidden)]
        pub fn main() {
            #call_stmt
        }
    }
}

struct ParsedFunction {
    fn_name: syn::Ident,
    is_async: bool,
}

fn parse_function(
    item: proc_macro2::TokenStream,
) -> (proc_macro2::TokenStream, Option<ParsedFunction>) {
    let mut item: syn::ItemFn = match syn::parse2(item.clone()) {
        Ok(item) => item,
        Err(_) => {
            // Very questionable recovery strategy: find the first instance of `main`
            // and replace it with `_main_impl` so that we can still compile with
            // fewer warnings.

            let mut seen_main = false;
            return (
                item.into_iter()
                    .map(|tt| match tt {
                        TokenTree::Ident(ident) if ident == "main" && !seen_main => {
                            seen_main = true;
                            TokenTree::Ident(Ident::new("_main_impl", Span::call_site()))
                        }
                        tt => tt,
                    })
                    .collect(),
                None,
            );
        }
    };

    let fn_name = quote::format_ident!("{}_impl", item.sig.ident);
    item.sig.ident = fn_name.clone();

    let is_async = item.sig.asyncness.is_some();

    (
        item.into_token_stream(),
        Some(ParsedFunction { fn_name, is_async }),
    )
}
