use proc_macro2::{Span, TokenStream};
use quote::quote;

use crate::api_project;

pub fn main_impl(
    item: TokenStream,
    ambient_toml: (Option<String>, String),
) -> anyhow::Result<TokenStream> {
    let mut item: syn::ItemFn = syn::parse2(item)?;
    let fn_name = quote::format_ident!("{}_impl", item.sig.ident);
    item.sig.ident = fn_name.clone();

    let is_async = item.sig.asyncness.is_some();

    let spans = Span::call_site();
    let mut path = syn::Path::from(syn::Ident::new("ambient_api", spans));
    path.leading_colon = Some(syn::Token![::](spans));
    let project_boilerplate = api_project::implementation(ambient_toml, path.clone(), false, true)?;

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

#[cfg(test)]
mod tests {
    use super::main_impl;
    use proc_macro2::TokenStream;
    use quote::quote;

    const AMBIENT_TOML: &str = r#"
    [project]
    name = "Test Project"
    id = "test_project"
    version = "0.0.1"
    "#;

    fn prelude() -> TokenStream {
        quote! {
            /// Auto-generated component definitions. These come from `ambient.toml` in the root of the project.
            pub mod components {
                use ::ambient_api::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
            }
            /// Auto-generated concept definitions. Concepts are collections of components that describe some form of gameplay concept.
            ///
            /// They do not have any runtime representation outside of the components that compose them.
            pub mod concepts {
                use super::components;
                use ::ambient_api::prelude::*;
            }
            /// Auto-generated message definitions. Messages are used to communicate between the client and serverside,
            /// as well as to other modules.
            pub mod messages {
                use ::ambient_api::{prelude::*, message::{Message, MessageSerde, MessageSerdeError}};
            }
        }
    }

    #[test]
    fn can_generate_impl_for_async_fn() {
        let body = quote! {
            pub async fn main() -> ResultEmpty {
                OkEmpty
            }
        };

        let prelude = prelude();

        let output = quote! {
            #prelude

            pub async fn main_impl() -> ResultEmpty {
                OkEmpty
            }

            #[no_mangle]
            #[doc(hidden)]
            pub fn main() {
                ::ambient_api::global::run_async(main_impl());
            }
        };

        assert_eq!(
            main_impl(body, (None, AMBIENT_TOML.to_owned()))
                .unwrap()
                .to_string(),
            output.to_string()
        );
    }

    #[test]
    fn can_generate_impl_for_sync_fn() {
        let body = quote! {
            pub fn main() -> ResultEmpty {
                OkEmpty
            }
        };

        let prelude = prelude();

        let output = quote! {
            #prelude

            pub fn main_impl() -> ResultEmpty {
                OkEmpty
            }

            #[no_mangle]
            #[doc(hidden)]
            pub fn main() {
                ::ambient_api::global::run_async(async { main_impl() });
            }
        };

        assert_eq!(
            main_impl(body, (None, AMBIENT_TOML.to_owned()))
                .unwrap()
                .to_string(),
            output.to_string()
        );
    }
}
