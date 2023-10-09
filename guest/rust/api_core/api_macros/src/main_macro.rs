use ambient_package_macro_common::{Context, RetrievableFile};
use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use quote::{format_ident, quote, ToTokens};
use syn::{Error, MethodTurbofish};

pub fn derive_main(item: TokenStream, ambient_toml: RetrievableFile) -> syn::Result<TokenStream> {
    let spans = Span::call_site();
    let mut path = syn::Path::from(syn::Ident::new("ambient_api", spans));
    path.leading_colon = Some(syn::Token![::](spans));

    let generated = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(anyhow::Error::new)
        .and_then(|rt| {
            rt.block_on(ambient_package_macro_common::generate_code(
                Some(ambient_toml),
                Context::GuestUser,
            ))
        })
        .map_err(|e| {
            let msg = format!(
                "Error while running Ambient package macro: {e}{}",
                e.source()
                    .map(|e| format!("\nCaused by: {e}"))
                    .unwrap_or_default()
            );

            Error::new(Span::call_site(), msg)
        })?;

    let package = generated
        .package
        .expect("guest code must be from a package");

    // Force package name to be a valid rust indent
    // Not sure if this is enforced in any step prior
    let mut package_name: String = package
        .manifest
        .package
        .name
        .chars()
        // Skip leading numbers
        // `0abc` is not a valid rust ident
        .take_while(|v| v.is_numeric())
        .filter_map(|v| match v {
            '-' => Some('_'),
            'a'..='z' => Some(v),
            '0'..='9' => Some(v),
            _ => None,
        })
        .collect();

    let package_id = package
        .manifest
        .package
        .id
        .map(|v| v.to_string())
        .unwrap_or_else(|| "unknown".into());

    let label = format_ident!("{package_name}_{package_id}",);

    let (item, parsed) = parse_function(item, format_ident!("{label}_main"));

    eprintln!("Generating entry point for {package_name:?}");

    /// Wrap the main function in an async shim
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

    let exec_name = format_ident!("{label}_exec");

    // let export_bindings = quote! {
    //     struct Guest;

    //     #path::__internal::export_bindings!(Guest);

    //     impl #path::__internal::Guest for Guest {
    //         fn init() {
    //             use #path::__internal::EXECUTOR;
    //             // once_cell::sync::Lazy::force(&EXECUTOR);
    //             main();
    //         }

    //         fn exec(source: #path::__internal::Source, message_name: String, message_data: Vec<u8>) {
    //             // exec_name(source, message_name, message_data)
    //         }
    //     }

    // };

    let boilerplate = generated.tokens;

    Ok(quote! {
        #item

        #boilerplate

        #[no_mangle]
        #[doc(hidden)]
        pub fn main() {
            // #export_bindings

            #call_stmt
        }


        // fn exec() {

        // }

        // fn #exec_name(source: #path::__internal::Source, message_name: String, message_data: Vec<u8>) {
        //     use #path::__internal::EXECUTOR;
        //     EXECUTOR.execute(source, message_name, message_data);
        // }
    })
}

struct ParsedFunction {
    fn_name: syn::Ident,
    is_async: bool,
}

/// Parses the annotated `async? fn main`,
fn parse_function(item: TokenStream, fn_name: Ident) -> (TokenStream, Option<ParsedFunction>) {
    let mut item: syn::ItemFn = match syn::parse2(item.clone()) {
        Ok(item) => item,
        // Triggered if there is a syntax error in the function body by the user
        //
        // TODO: consider reporting a spanned error instead
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
                            // Could this cause RA issues, as main is missing, rugpulling it
                            TokenTree::Ident(Ident::new("_main_impl", Span::call_site()))
                        }
                        tt => tt,
                    })
                    .collect(),
                None,
            );
        }
    };

    item.sig.ident = fn_name.clone();

    let is_async = item.sig.asyncness.is_some();

    (
        item.into_token_stream(),
        Some(ParsedFunction { fn_name, is_async }),
    )
}
