use super::{
    identifier::{IdentifierPath, IdentifierPathBuf},
    tree::{Tree, TreeNode, TreeNodeInner},
};
use anyhow::Context;
use quote::quote;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Component {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub type_: ComponentType,
    #[serde(default)]
    pub attributes: Vec<String>,
    #[serde(default)]
    pub default: Option<toml::Value>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ComponentType {
    String(String),
    ContainerType {
        #[serde(rename = "type")]
        type_: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        element_type: Option<String>,
    },
}
impl ComponentType {
    pub fn to_token_stream(
        &self,
        api_name: &syn::Path,
        fully_qualified: bool,
    ) -> anyhow::Result<proc_macro2::TokenStream> {
        match self {
            ComponentType::String(ty) => {
                convert_primitive_type_to_rust_type(ty, api_name, fully_qualified)
                    .context("invalid primitive type")
            }
            ComponentType::ContainerType {
                type_,
                element_type,
            } => {
                if let Some(element_type) = element_type {
                    let container_ty = convert_container_type_to_rust_type(type_)
                        .context("invalid container type")?;

                    let element_ty = convert_primitive_type_to_rust_type(
                        element_type,
                        api_name,
                        fully_qualified,
                    )
                    .context("invalid element type")?;

                    Ok(quote! { #container_ty < #element_ty > })
                } else {
                    Ok(
                        convert_primitive_type_to_rust_type(type_, api_name, fully_qualified)
                            .context("invalid primitive type")?,
                    )
                }
            }
        }
    }
}

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
            let component_ty = component.type_.to_token_stream(api_name, true)?;

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

fn convert_primitive_type_to_rust_type(
    ty: &str,
    api_name: &syn::Path,
    fully_qualified: bool,
) -> Option<proc_macro2::TokenStream> {
    let fully_qualified_prefix = if fully_qualified {
        quote! { #api_name::global:: }
    } else {
        quote! {}
    };

    match ty {
        "Empty" => Some(quote! {()}),
        "Bool" => Some(quote! {bool}),
        "EntityId" => Some(quote! {#fully_qualified_prefix EntityId}),
        "F32" => Some(quote! {f32}),
        "F64" => Some(quote! {f64}),
        "Mat4" => Some(quote! {#fully_qualified_prefix Mat4}),
        "I32" => Some(quote! {i32}),
        "Quat" => Some(quote! {#fully_qualified_prefix Quat}),
        "String" => Some(quote! {String}),
        "U32" => Some(quote! {u32}),
        "U64" => Some(quote! {u64}),
        "Vec2" => Some(quote! {#fully_qualified_prefix Vec2}),
        "Vec3" => Some(quote! {#fully_qualified_prefix Vec3}),
        "Vec4" => Some(quote! {#fully_qualified_prefix Vec4}),
        "Uvec2" => Some(quote! {#fully_qualified_prefix UVec2}),
        "Uvec3" => Some(quote! {#fully_qualified_prefix UVec3}),
        "Uvec4" => Some(quote! {#fully_qualified_prefix UVec4}),
        _ => None,
    }
}

fn convert_container_type_to_rust_type(ty: &str) -> Option<proc_macro2::TokenStream> {
    match ty {
        "Vec" => Some(quote! {Vec}),
        "Option" => Some(quote! {Option}),
        _ => None,
    }
}
