use ambient_package_macro_common::{Context, RetrievableFile};
use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use quote::{quote, ToTokens};

pub fn main(item: TokenStream, ambient_toml: RetrievableFile) -> TokenStream {
    let (item, parsed) = parse_function(item);

    let spans = Span::call_site();
    let mut path = syn::Path::from(syn::Ident::new("ambient_api", spans));
    path.leading_colon = Some(syn::Token![::](spans));

    let boilerplate = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(anyhow::Error::new)
        .and_then(|rt| {
            rt.block_on(ambient_package_macro_common::generate_code(
                Some(ambient_toml),
                Context::GuestUser,
            ))
        })
        .unwrap_or_else(|e| {
            let msg = format!(
                "Error while running Ambient package macro: {e}{}",
                e.source()
                    .map(|e| format!("\nCaused by: {e}"))
                    .unwrap_or_default()
            );
            quote::quote! {
                compile_error!(#msg);
            }
        });

    let call_stmt = if let Some(ParsedFunction { fn_name, is_async }) = parsed {
        let call_expr = if is_async {
            quote! { #fn_name(world) }
        } else {
            quote! { async { #fn_name(world) } }
        };

        quote! { #path::global::run_async(#call_expr) }
    } else {
        quote! {}
    };

    quote! {
        #item

        #boilerplate

        #[no_mangle]
        #[doc(hidden)]
        pub fn main(world: &mut dyn #path::ecs::World) {
            #call_stmt
        }
    }
}

struct ParsedFunction {
    fn_name: syn::Ident,
    is_async: bool,
}

fn parse_function(item: TokenStream) -> (TokenStream, Option<ParsedFunction>) {
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
