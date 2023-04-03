extern crate proc_macro;

use ambient_project::{IdentifierPathBuf, Manifest};
use quote::quote;

use tree::Tree;

#[cfg(test)]
mod tests;

mod component;
mod concept;
mod message;
mod tree;
mod util;

pub const MANIFEST: &str = include_str!("../../../ambient.toml");

pub enum Context {
    Host,
    Guest {
        api_path: syn::Path,
        fully_qualified_path: bool,
    },
}

pub fn implementation(
    (file_path, contents): (Option<String>, String),
    context: Context,
    is_api_manifest: bool,
    validate_namespaces_documented: bool,
) -> anyhow::Result<proc_macro2::TokenStream> {
    let manifest: Manifest = toml::from_str(&contents)?;
    let project_path = if !is_api_manifest {
        manifest.project_path()
    } else {
        IdentifierPathBuf::empty()
    };

    let component_tree = Tree::new(&manifest.components, validate_namespaces_documented)?;
    let components_tokens =
        component::tree_to_token_stream(&component_tree, &context, project_path.as_path())?;

    let concept_tree = Tree::new(&manifest.concepts, validate_namespaces_documented)?;
    let concept_tokens = concept::tree_to_token_stream(&concept_tree, &component_tree, &context)?;

    let message_tree = Tree::new(&manifest.messages, validate_namespaces_documented)?;
    let message_tokens = message::tree_to_token_stream(&message_tree, &context, is_api_manifest)?;

    let manifest = file_path.map(
        |file_path| quote! { const _PROJECT_MANIFEST: &'static str = include_str!(#file_path); },
    );
    Ok(quote!(
        #manifest
        /// Auto-generated component definitions. These come from `ambient.toml` in the root of the project.
        pub mod components {
            #components_tokens
        }
        /// Auto-generated concept definitions. Concepts are collections of components that describe some form of gameplay concept.
        ///
        /// They do not have any runtime representation outside of the components that compose them.
        pub mod concepts {
            #concept_tokens
        }
        /// Auto-generated message definitions. Messages are used to communicate between the client and serverside,
        /// as well as to other modules.
        pub mod messages {
            #message_tokens
        }
    ))
}
