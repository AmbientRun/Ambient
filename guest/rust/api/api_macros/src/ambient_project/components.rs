use super::{
    identifier::{IdentifierPath, IdentifierPathBuf},
    manifest::Component,
    tree::{Tree, TreeNode, TreeNodeInner},
};
use quote::quote;

pub fn tree_to_token_stream(
    tree: &Tree<Component>,
    api_name: &syn::Path,
    project_path: IdentifierPath,
) -> anyhow::Result<proc_macro2::TokenStream> {
    to_token_stream(tree.root(), api_name, project_path)
}

fn to_token_stream(
    node: &TreeNode<Component>,
    api_name: &syn::Path,
    project_path: IdentifierPath,
) -> anyhow::Result<proc_macro2::TokenStream> {
    let name = node.path.last().map(|s| s.as_ref()).unwrap_or_default();
    match &node.inner {
        TreeNodeInner::Namespace(ns) => {
            let children = ns
                .children
                .values()
                .map(|child| to_token_stream(child, api_name, project_path))
                .collect::<Result<Vec<_>, _>>()?;

            let prelude = quote! {
                use #api_name::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
            };

            Ok(if name.is_empty() {
                quote! {
                    #prelude
                    #(#children)*
                }
            } else {
                let name_ident: syn::Path = syn::parse_str(name)?;
                let doc_comment_fragment = ns.namespace.as_ref().map(|n| {
                    let mut doc_comment = format!("**{}**", n.name);
                    if !n.description.is_empty() {
                        doc_comment += &format!(": {}", n.description.replace('\n', "\n\n"));
                    }

                    quote! {
                        #[doc = #doc_comment]
                    }
                });
                quote! {
                    #doc_comment_fragment
                    pub mod #name_ident {
                        #prelude
                        #(#children)*
                    }
                }
            })
        }
        TreeNodeInner::Other(component) => {
            let name_ident: syn::Path = syn::parse_str(name)?;
            let name_uppercase_ident: syn::Path = syn::parse_str(&name.to_ascii_uppercase())?;
            let component_ty = component.type_.to_token_stream(api_name)?;

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
        }
    }
}
