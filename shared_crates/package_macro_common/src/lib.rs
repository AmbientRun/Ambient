extern crate proc_macro;

pub use ambient_package_semantic::RetrievableFile;
use ambient_package_semantic::{Item, ItemId, ItemMap, Package, Scope, Semantic, Type, TypeInner};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::collections::HashMap;

mod assets;
mod components;
mod concepts;
mod context;
mod enums;
mod messages;

pub use context::Context;

pub async fn generate_code(
    manifest: Option<RetrievableFile>,
    context: context::Context,
) -> anyhow::Result<TokenStream> {
    let mut semantic = Semantic::new(false).await?;

    let package_id = if let Some(manifest) = manifest {
        Some(semantic.add_package(manifest, None).await?)
    } else {
        None
    };

    semantic.resolve()?;

    let items = &semantic.items;
    let root_scope = &*semantic.root_scope();
    let type_printer = {
        let mut map = HashMap::new();
        for type_id in root_scope.types.values() {
            let type_ = items.get(*type_id);
            if let TypeInner::Primitive(pt) = type_.inner {
                let ty_tokens = syn::parse_str::<syn::Type>(&pt.to_string())?.to_token_stream();
                map.insert(*type_id, ty_tokens.clone());
                map.insert(items.get_vec_id(*type_id), quote! {Vec::<#ty_tokens>});
                map.insert(items.get_option_id(*type_id), quote! {Option::<#ty_tokens>});
            }
        }
        TypePrinter(map)
    };

    let outputs = semantic
        .packages
        .values()
        .map(|id| generate_package(context, items, &type_printer, *id))
        .collect::<Result<Vec<_>, _>>()?;

    let imports = if let Some(package_id) = package_id {
        let package = items.get(package_id);

        let dependencies = std::iter::once(anyhow::Ok(("this", package.data.id.to_string())))
            .chain(package.dependencies.iter().map(|(alias, dependency)| {
                let dependency = items.get(dependency.id);
                anyhow::Ok((alias.as_str(), dependency.data.id.to_string()))
            }))
            .collect::<Result<Vec<_>, _>>()?;

        let dependencies = dependencies.into_iter().map(|(alias, raw)| {
            let alias = make_path(alias);
            let raw = make_path(&raw);
            quote! {
                pub use raw::#raw as #alias;
            }
        });

        Some(quote! {
            #(#dependencies)*
        })
    } else {
        None
    };

    let output = quote! {
        mod raw {
            #(#outputs)*
        }

        #imports
    };

    let output = if context == Context::GuestUser {
        // In guest code, we wrap all generated output in an `packages` module to avoid polluting their
        // global scope.
        quote! {
            pub mod packages {
                #output
            }
        }
    } else {
        output
    };

    Ok(output)
}

fn generate_package(
    context: context::Context,
    items: &ItemMap,
    type_printer: &TypePrinter,
    package_id: ItemId<Package>,
) -> anyhow::Result<TokenStream> {
    let package = items.get(package_id);
    let generate_from_scope = &*items.get(package.scope_id);

    let entity = match context {
        Context::GuestUser => {
            let package_ambient_id = package.data().id.to_string();
            quote! {
                pub fn entity() -> ambient_api::global::EntityId {
                    use ambient_api::once_cell::sync::Lazy;
                    static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                        ambient_api::package::get_entity_for_package_id(#package_ambient_id).expect("Failed to get package entity - was it despawned?")
                    });
                    *ENTITY
                }
            }
        }
        _ => TokenStream::new(),
    };
    let generated_output = generate_scope(context, items, type_printer, generate_from_scope)?;
    let components_init = components::generate_init(context, items, generate_from_scope)?;

    let id = make_path(package.data.id.as_str());
    anyhow::Ok(quote! {
        pub mod #id {
            #entity
            #generated_output
            #components_init
        }
    })
}

fn generate_scope(
    context: context::Context,
    items: &ItemMap,
    type_printer: &TypePrinter,
    scope: &Scope,
) -> anyhow::Result<TokenStream> {
    let scopes = scope
        .scopes
        .values()
        .map(|s| {
            let scope = items.get(*s);
            if !context.should_generate(scope.data()) {
                return Ok(quote! {});
            }

            let id = make_path(scope.data.id.as_str());
            let inner = generate_scope(context, items, type_printer, &scope)?;
            if inner.is_empty() {
                return Ok(quote! {});
            }

            Ok(quote! {
                #[allow(unused)]
                pub mod #id {
                    #inner
                }
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let components = components::generate(context, items, type_printer, scope)?;
    let concepts = concepts::generate(context, items, type_printer, scope)?;
    let messages = messages::generate(context, items, type_printer, scope)?;
    let types = enums::generate(context, items, scope)?;
    let assets = assets::generate(context, items, scope)?;

    Ok(quote! {
        #(#scopes)*

        #components
        #concepts
        #messages
        #types
        #assets
    })
}

fn make_path(id: &str) -> syn::Path {
    syn::parse_str(id).unwrap()
}

pub struct TypePrinter(HashMap<ItemId<Type>, TokenStream>);
impl TypePrinter {
    pub fn get(
        &self,
        context: context::Context,
        items: &ItemMap,
        prefix: Option<&str>,
        id: ItemId<Type>,
    ) -> anyhow::Result<TokenStream> {
        match self.0.get(&id) {
            Some(ts) => Ok(ts.clone()),
            None => context.get_path(items, prefix, id),
        }
    }
}
