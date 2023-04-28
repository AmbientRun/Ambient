extern crate proc_macro;

use ambient_project::{IdentifierPathBuf, Manifest};
use quote::quote;

use proc_macro2::Ident;
use std::path::PathBuf;
use tree::Tree;

#[cfg(test)]
mod tests;

mod component;
mod concept;
mod message;
mod tree;
mod util;

pub enum Context {
    Host,
    Guest {
        api_path: syn::Path,
        fully_qualified_path: bool,
    },
}

pub enum ManifestSource {
    Path(PathBuf),
    String(String),
}
impl ManifestSource {
    fn build(&self) -> anyhow::Result<(Manifest, proc_macro2::TokenStream)> {
        match self {
            Self::Path(file_path) => {
                let manifest = Manifest::from_file(file_path)?;
                let mut file_paths = vec![file_path.to_str().unwrap().to_string()];
                let dir = file_path.parent().unwrap();
                for include in &manifest.project.includes {
                    let path = dir.join(include);
                    let file_path = path.to_str().unwrap().to_string();
                    file_paths.push(file_path);
                }
                let force_reload = file_paths.into_iter().enumerate().map(|(i, file_path)| {
                    let name = Ident::new(
                        &format!("_PROJECT_MANIFEST_{}", i),
                        proc_macro2::Span::call_site(),
                    );
                    quote! { const #name: &'static str = include_str!(#file_path); }
                });
                Ok((manifest, quote! { #(#force_reload)* }))
            }
            Self::String(string) => Ok((Manifest::parse(string)?, quote! {})),
        }
    }
}

pub fn generate_code(
    manifest: ManifestSource,
    context: Context,
    is_api_manifest: bool,
    validate_namespaces_documented: bool,
) -> anyhow::Result<proc_macro2::TokenStream> {
    let (manifest, force_reload) = manifest.build()?;

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

    Ok(quote!(
        #force_reload

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
        /// Auto-generated message definitions. Messages are used to communicate with the runtime, the other side of the network,
        /// and with other modules.
        pub mod messages {
            #message_tokens
        }
    ))
}
