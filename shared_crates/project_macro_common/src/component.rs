use super::{
    tree::{Tree, TreeNode},
    util,
};
use ambient_project::{Component, IdentifierPath, IdentifierPathBuf};
use proc_macro2::TokenStream;
use quote::quote;

pub fn tree_to_token_stream(
    tree: &Tree<Component>,
    api_name: &syn::Path,
    project_path: IdentifierPath,
) -> anyhow::Result<proc_macro2::TokenStream> {
    to_token_stream(
        tree.root(),
        api_name,
        &quote! {
            use #api_name::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
        },
        project_path,
    )
}

fn to_token_stream(
    node: &TreeNode<Component>,
    api_path: &syn::Path,
    prelude: &TokenStream,
    project_path: IdentifierPath,
) -> anyhow::Result<TokenStream> {
    util::tree_to_token_stream(
        node,
        api_path,
        prelude,
        |node, api_path, prelude| to_token_stream(node, api_path, prelude, project_path),
        |name, component, api_path| {
            let name_ident: syn::Path = syn::parse_str(name)?;
            let name_uppercase_ident: syn::Path = syn::parse_str(&name.to_ascii_uppercase())?;
            let component_ty = component.type_.to_token_stream(api_path, true, false)?;

            let mut doc_comment = format!("**{}**", component.name);

            if !component.description.is_empty() {
                doc_comment += &format!(": {}", component.description.replace('\n', "\n\n"));
            }

            // Metadata
            if !component.attributes.is_empty() {
                doc_comment += &format!(
                    "\n\n*Attributes*: {}",
                    component.attributes.clone().join(", ")
                )
            }
            if let Some(default) = component.default.as_ref() {
                doc_comment += &format!("\n\n*Suggested Default*: {default}")
            }

            let id =
                IdentifierPathBuf::from_iter(project_path.iter().chain(node.path.iter()).cloned())
                    .to_string();
            let doc_comment = doc_comment.trim();

            Ok(quote! {
                static #name_uppercase_ident: Lazy< Component< #component_ty > > = Lazy::new(|| __internal_get_component(#id));
                #[doc = #doc_comment]
                pub fn #name_ident() -> Component< #component_ty > { *#name_uppercase_ident }
            })
        },
    )
}
