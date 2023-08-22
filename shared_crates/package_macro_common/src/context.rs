use std::cell::Ref;

use ambient_package_semantic::{Item, ItemData, ItemId, ItemMap, ItemSource, ItemType};
use proc_macro2::TokenStream;
use quote::quote;

use crate::make_path;

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
        let item = items.get(id);
        if *self == Context::GuestUser && item.data().source != ItemSource::User {
            return None;
        }
        Some(item)
    }

    pub fn get_path<T: Item>(
        &self,
        items: &ItemMap,
        prefix: Option<&str>,
        id: ItemId<T>,
    ) -> anyhow::Result<TokenStream> {
        let item = items.get(id);
        let path_prefix = self.path_prefix_impl(item.data());
        let type_namespace = match T::TYPE {
            ItemType::Component => "components::",
            ItemType::Concept => "concepts::",
            ItemType::Message => "messages::",
            ItemType::Type => "types::",
            ItemType::Attribute => "attributes::",
            ItemType::Scope => "scopes::",
            ItemType::Package => "packages::",
        };
        let prefix = format!("{type_namespace}{}", prefix.unwrap_or_default());
        let path =
            make_path(&items.fully_qualified_display_path(&*item, None, Some(prefix.as_str())));

        Ok(quote! { #path_prefix #path })
    }

    pub(crate) fn path_prefix_impl(&self, data: &ItemData) -> TokenStream {
        match (self, data.source) {
            (_, ItemSource::System) => quote! {},

            (Context::Host, ItemSource::Ambient) => quote! { crate::generated::raw:: },
            (Context::GuestApi, ItemSource::Ambient) => quote! { crate:: },

            (Context::GuestApi | Context::Host, ItemSource::User) => {
                unreachable!("user items should not be in api or host scope")
            }

            (Context::GuestUser, ItemSource::Ambient) => quote! { ambient_api::core:: },
            (Context::GuestUser, ItemSource::User) => quote! { crate::packages::raw:: },
        }
    }

    pub fn should_generate(&self, data: &ItemData) -> bool {
        match (self, data.source) {
            (_, ItemSource::System) => false,
            (Context::Host, ItemSource::Ambient) => true,
            (Context::Host, ItemSource::User) => {
                unreachable!("user items should not be in host scope")
            }
            (Context::GuestApi, ItemSource::Ambient) => true,
            (Context::GuestApi, ItemSource::User) => {
                unreachable!("user items should not be in api scope")
            }
            (Context::GuestUser, ItemSource::Ambient) => false,
            (Context::GuestUser, ItemSource::User) => true,
        }
    }
}
