extern crate proc_macro;

use ambient_project::ItemPathBuf;
use ambient_project_semantic::{
    ArrayFileProvider, Item, ItemData, ItemId, ItemMap, ItemSource, ItemType, Scope, Semantic,
    Type, TypeInner,
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::{cell::Ref, collections::HashMap, path::Path};

mod components;
mod concepts;
mod enums;
mod messages;
mod scopes;

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

    pub fn get_path<T: Item>(
        &self,
        items: &ItemMap,
        prefix: Option<&str>,
        root_scope_id: ItemId<Scope>,
        id: ItemId<T>,
    ) -> anyhow::Result<TokenStream> {
        let item = items.get(id).unwrap();
        let path_prefix = self.path_prefix_impl(item.data());
        let type_namespace = match T::TYPE {
            ItemType::Component => "components::",
            ItemType::Concept => "concepts::",
            ItemType::Message => "messages::",
            ItemType::Type => "types::",
            ItemType::Attribute => "attributes::",
            ItemType::Scope => "scopes::",
        };
        let prefix = format!("{type_namespace}{}", prefix.unwrap_or_default());
        let path = make_path(&items.fully_qualified_display_path(
            &*item,
            Some(root_scope_id),
            Some(prefix.as_str()),
        )?);

        Ok(quote! { #path_prefix #path })
    }

    fn path_prefix_impl(&self, data: &ItemData) -> TokenStream {
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

    let scopes = scopes::make_scopes(
        context,
        items,
        &type_printer,
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

pub struct TypePrinter(HashMap<ItemId<Type>, TokenStream>);
impl TypePrinter {
    pub fn get(
        &self,
        context: Context,
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
