use std::path::PathBuf;

use anyhow::Context;
use quote::quote;

use self::identifier::IdentifierPathBuf;

#[cfg(test)]
mod tests;

mod components;
mod concepts;
mod identifier;
mod manifest;

pub fn read_file(file_path: String) -> anyhow::Result<(Option<String>, String)> {
    let file_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").context("no manifest dir")?)
        .join(file_path);
    let file_path_str = format!("{}", file_path.display());

    let contents = std::fs::read_to_string(&file_path)?;

    Ok((Some(file_path_str), contents))
}

pub fn implementation(
    (file_path, contents): (Option<String>, String),
    api_name: syn::Path,
    global_namespace: bool,
    validate_namespaces_documented: bool,
) -> anyhow::Result<proc_macro2::TokenStream> {
    let manifest: manifest::Manifest = toml::from_str(&contents)?;
    let project_path = if !global_namespace {
        manifest.project_path()
    } else {
        IdentifierPathBuf::empty()
    };

    let tree = components::Tree::new(&manifest, validate_namespaces_documented)?;
    let components_tokens = tree.to_token_stream(&api_name, project_path.as_path())?;
    let concepts = concepts::generate_tokens(&manifest, &tree, &api_name)?;

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
        pub mod concepts {
            #concepts
        }
    ))
}
