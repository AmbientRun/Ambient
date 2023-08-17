extern crate proc_macro;

use ambient_ember::ItemPathBuf;
use ambient_ember_semantic::{
    ArrayFileProvider, Item, ItemId, ItemMap, ItemSource, Scope, Semantic, Type, TypeInner,
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::{collections::HashMap, path::Path};

mod assets;
mod components;
mod concepts;
mod context;
mod enums;
mod messages;

pub use context::Context;

pub enum ManifestSource<'a> {
    Path { ember_path: &'a Path },
    Array(&'a [(&'a str, &'a str)]),
}

pub async fn generate_code(
    manifest: Option<ManifestSource<'_>>,
    context: context::Context,
    generate_from_scope_path: Option<&str>,
) -> anyhow::Result<TokenStream> {
    let mut semantic = Semantic::new().await?;

    if let Some(manifest) = manifest {
        match manifest {
            ManifestSource::Path { ember_path } => semantic.add_ember(ember_path).await,
            ManifestSource::Array(files) => {
                semantic
                    .add_file(
                        Path::new("ambient.toml"),
                        &ArrayFileProvider { files },
                        ItemSource::User,
                        None,
                    )
                    .await
            }
        }?;
    }

    semantic.resolve()?;

    let items = &semantic.items;
    let root_scope = &*semantic.root_scope();
    let type_printer = {
        let mut map = HashMap::new();
        for type_id in root_scope.types.values() {
            let type_ = items.get(*type_id).expect("type id not in items");
            if let TypeInner::Primitive(pt) = type_.inner {
                let ty_tokens = syn::parse_str::<syn::Type>(&pt.to_string())?.to_token_stream();
                map.insert(*type_id, ty_tokens.clone());
                map.insert(items.get_vec_id(*type_id), quote! {Vec::<#ty_tokens>});
                map.insert(items.get_option_id(*type_id), quote! {Option::<#ty_tokens>});
            }
        }
        TypePrinter(map)
    };

    let generate_from_scope_id = generate_from_scope_path
        .map(|id| ItemPathBuf::new(id).expect("invalid generate_from_scope_path"))
        .map(|id| {
            items
                .get_scope_id(
                    semantic.root_scope_id,
                    id.as_path().scope_iter().expect(
                        "invalid generate_from_scope_path: the last element must be a scope",
                    ),
                )
                .unwrap()
        })
        .unwrap_or(semantic.root_scope_id);
    let generate_from_scope = &*items.get(generate_from_scope_id)?;

    let generated_output = generate(
        context,
        items,
        &type_printer,
        generate_from_scope_id,
        generate_from_scope,
    )?;

    let components_init =
        components::generate_init(context, items, generate_from_scope_id, generate_from_scope)?;

    let output = quote! {
        #generated_output
        #components_init
    };

    let output = if context == Context::GuestUser {
        // In guest code, we wrap all generated output in an `embers` module to avoid polluting their
        // global scope.
        quote! {
            pub mod embers {
                #output
            }
        }
    } else {
        output
    };

    Ok(output)
}

fn generate(
    context: context::Context,
    items: &ItemMap,
    type_printer: &TypePrinter,
    root_scope_id: ItemId<Scope>,
    scope: &Scope,
) -> anyhow::Result<TokenStream> {
    let scopes = scope
        .scopes
        .values()
        .map(|s| {
            let scope = items.get(*s)?;
            if !context.should_generate(scope.data()) {
                return Ok(quote! {});
            }

            let id = make_path(scope.data.id.as_str());
            let inner = generate(context, items, type_printer, root_scope_id, &scope)?;
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

    let components = components::generate(context, items, type_printer, root_scope_id, scope)?;
    let concepts = concepts::generate(context, items, type_printer, root_scope_id, scope)?;
    let messages = messages::generate(context, items, type_printer, root_scope_id, scope)?;
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
        root_scope_id: ItemId<Scope>,
        id: ItemId<Type>,
    ) -> anyhow::Result<TokenStream> {
        match self.0.get(&id) {
            Some(ts) => Ok(ts.clone()),
            None => context.get_path(items, prefix, root_scope_id, id),
        }
    }
}
