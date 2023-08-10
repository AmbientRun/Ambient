use ambient_project_semantic::{ItemMap, Scope};
use proc_macro2::TokenStream;
use quote::quote;

use crate::Context;

pub fn generate(context: Context, _items: &ItemMap, scope: &Scope) -> anyhow::Result<TokenStream> {
    let Some(api_path) = (context == Context::GuestUser)
        .then_some(())
        .and_then(|_| context.guest_api_path()) else { return Ok(quote!{}) };

    let ember_id = scope.original_id.to_string();
    if ember_id.is_empty() {
        return Ok(quote! {});
    }

    Ok(quote! {
        /// Helpers for accessing the assets for this ember.
        pub mod assets {
            pub fn url(path: &str) -> String {
                #api_path::asset::url_for_ember_asset(#ember_id, path).unwrap()
            }
        }
    })
}
