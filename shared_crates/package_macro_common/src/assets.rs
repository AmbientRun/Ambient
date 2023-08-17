use ambient_package_semantic::{ItemMap, Scope};
use proc_macro2::TokenStream;
use quote::quote;

use crate::Context;

pub fn generate(context: Context, _items: &ItemMap, scope: &Scope) -> anyhow::Result<TokenStream> {
    let Some(api_path) = (context == Context::GuestUser)
        .then_some(())
        .and_then(|_| context.guest_api_path()) else { return Ok(quote!{}) };

    let package_id = scope.original_id.to_string();
    if package_id.is_empty() {
        return Ok(quote! {});
    }

    Ok(quote! {
        /// Helpers for accessing the assets for this package.
        pub mod assets {
            pub fn url(path: &str) -> String {
                #api_path::asset::url_for_package_asset(#package_id, path).unwrap()
            }
        }
    })
}
