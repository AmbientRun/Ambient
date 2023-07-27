extern crate proc_macro;

use ambient_project::ItemPathBuf;
use ambient_project_semantic::{
    ArrayFileProvider, Item, ItemData, ItemId, ItemMap, ItemSource, Semantic, TypeInner,
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::{cell::Ref, collections::HashMap, path::Path};

mod components;
mod concepts;
mod messages;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Context {
    /// Generating for the Ambient host. Use host definitions.
    Host,
    /// Generating for the Ambient Rust API. Use guest definitions relative to itself.
    GuestApi,
    /// Generating for Ambient Rust guest code. Use guest definitions relative to the API.
    GuestUser,
}
impl Context {
    pub fn guest_api_path(&self) -> Option<TokenStream> {
        match self {
            Context::Host => None,
            Context::GuestApi => Some(quote! {crate}),
            Context::GuestUser => Some(quote! {ambient_api}),
        }
    }

    pub fn extract_item_if_relevant<'a, T: Item>(
        &self,
        items: &'a ItemMap,
        id: ItemId<T>,
    ) -> Option<Ref<'a, T>> {
        let item = items.get(id).unwrap();
        if *self == Context::GuestUser && item.data().source != ItemSource::User {
            return None;
        }
        Some(item)
    }

    pub fn path_prefix(&self, data: &ItemData) -> TokenStream {
        match (self, data.source) {
            (_, ItemSource::System) => quote! {},

            (Context::Host, ItemSource::Ambient) => quote! { crate::generated:: },
            (Context::GuestApi, ItemSource::Ambient) => quote! { crate:: },

            (Context::GuestApi | Context::Host, ItemSource::User) => {
                unreachable!("user items should not be in api or host scope")
            }

            (Context::GuestUser, ItemSource::Ambient) => quote! { ambient_api::core:: },
            (Context::GuestUser, ItemSource::User) => quote! { crate:: },
        }
    }
}

pub enum ManifestSource<'a> {
    Path { ember_path: &'a Path },
    Array(&'a [(&'a str, &'a str)]),
}

pub fn generate_code(
    manifest: Option<ManifestSource<'_>>,
    context: Context,
    generate_from_scope_path: Option<&str>,
) -> anyhow::Result<TokenStream> {
    let mut semantic = Semantic::new()?;
    semantic.add_ambient_schema()?;

    if let Some(manifest) = manifest {
        match manifest {
            ManifestSource::Path { ember_path } => semantic.add_ember(ember_path),
            ManifestSource::Array(files) => semantic.add_file(
                Path::new("ambient.toml"),
                &ArrayFileProvider { files },
                ItemSource::User,
            ),
        }?;
    }

    // let mut printer = ambient_project_semantic::Printer::new();
    semantic.resolve()?;
    // printer.print(&semantic)?;

    let items = &semantic.items;
    let root_scope = &*semantic.root_scope();
    let type_map = {
        let mut type_map = HashMap::new();

        // First pass: add all root-level primitive types
        for type_id in root_scope.types.values() {
            let type_ = items.get(*type_id).expect("type id not in items");
            if let TypeInner::Primitive(pt) = type_.inner {
                let ty_tokens = syn::parse_str::<syn::Type>(&pt.to_string())?.to_token_stream();
                type_map.insert(*type_id, ty_tokens.clone());
                type_map.insert(items.get_vec_id(*type_id), quote! {Vec::<#ty_tokens>});
                type_map.insert(items.get_option_id(*type_id), quote! {Option::<#ty_tokens>});
            }
        }

        // Second pass: traverse the type graph and add all enums
        root_scope.visit_recursive(items, |scope| {
            for type_id in scope.types.values() {
                let type_ = items.get(*type_id).expect("type id not in items");
                if let TypeInner::Enum { .. } = type_.inner {
                    type_map.insert(*type_id, quote! {u32});
                }
            }
            Ok(())
        })?;

        type_map
    };

    let generate_from_scope_id = generate_from_scope_path
        .map(|id| ItemPathBuf::new(id).expect("invalid generate_from_scope_path"))
        .map(|id| {
            items
                .get_scope_id(semantic.root_scope_id, id.as_path().iter())
                .unwrap()
        })
        .unwrap_or(semantic.root_scope_id);
    let generate_from_scope = &*items.get(generate_from_scope_id)?;

    let scopes = scopes::make_scopes(
        context,
        items,
        &type_map,
        generate_from_scope_id,
        generate_from_scope,
    )?;

    let components_init =
        components::make_init(context, items, generate_from_scope_id, generate_from_scope)?;

    let output = quote! {
        #scopes
        #components_init
    };

    // println!("{}", output.to_string());

    Ok(output)
}

fn make_path(id: &str) -> syn::Path {
    syn::parse_str(id).unwrap()
}
mod scopes {
    use std::collections::HashMap;

    use ambient_project_semantic::{ItemId, ItemMap, Scope, Type};
    use proc_macro2::TokenStream;
    use quote::quote;

    use crate::{make_path, Context};

    pub fn make_scopes(
        context: Context,
        items: &ItemMap,
        type_map: &HashMap<ItemId<Type>, TokenStream>,
        root_scope_id: ItemId<Scope>,
        scope: &Scope,
    ) -> anyhow::Result<TokenStream> {
        let scopes = scope
            .scopes
            .values()
            .map(|s| {
                let scope = items.get(*s)?;
                let id = make_path(&scope.data.id.as_snake_case());
                let inner = make_scopes(context, items, type_map, root_scope_id, &scope)?;
                if !inner.is_empty() {
                    Ok(quote! {
                        #[allow(unused)]
                        pub mod #id {
                            #inner
                        }
                    })
                } else {
                    Ok(quote! {})
                }
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        let components = {
            let inner = crate::components::make_definitions(
                context,
                items,
                &type_map,
                root_scope_id,
                scope,
            )?;
            if inner.is_empty() {
                quote! {}
            } else {
                quote! {
                    /// Auto-generated component definitions.
                    pub mod components {
                        #inner
                    }
                }
            }
        };
        let concepts = {
            let inner =
                crate::concepts::make_definitions(context, items, &type_map, root_scope_id, scope)?;
            if inner.is_empty() {
                quote! {}
            } else {
                quote! {
                    /// Auto-generated concept definitions. Concepts are collections of components that describe some form of gameplay concept.
                    ///
                    /// They do not have any runtime representation outside of the components that compose them.
                    pub mod concepts {
                        #inner
                    }
                }
            }
        };
        let messages = {
            let inner = crate::messages::make_definitions(context, items, &type_map, scope)?;
            if inner.is_empty() {
                quote! {}
            } else {
                quote! {
                    /// Auto-generated message definitions. Messages are used to communicate with the runtime, the other side of the network,
                    /// and with other modules.
                    pub mod messages {
                        #inner
                    }
                }
            }
        };

        Ok(quote! {
            #(#scopes)*

            #components
            #concepts
            #messages
        })
    }
}
