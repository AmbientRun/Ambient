use ambient_project_macro_common::{Context, ManifestSource};
use proc_macro2::{Span, TokenStream};
use quote::quote;

pub fn main_impl(item: TokenStream, ambient_toml: ManifestSource) -> anyhow::Result<TokenStream> {
    let mut item: syn::ItemFn = syn::parse2(item)?;
    let fn_name = quote::format_ident!("{}_impl", item.sig.ident);
    item.sig.ident = fn_name.clone();

    let is_async = item.sig.asyncness.is_some();

    let spans = Span::call_site();
    let mut path = syn::Path::from(syn::Ident::new("ambient_api", spans));
    path.leading_colon = Some(syn::Token![::](spans));

    let project_boilerplate =
        ambient_project_macro_common::generate_code(Some(ambient_toml), Context::GuestUser, None)?;

    let call_expr = if is_async {
        quote! { #fn_name() }
    } else {
        quote! { async { #fn_name() } }
    };

    Ok(quote! {
        #project_boilerplate

        #item

        #[no_mangle]
        #[doc(hidden)]
        pub fn main() {
            #path::global::run_async(#call_expr);
        }
    })
}
