use super::{
    tree::{Tree, TreeNode},
    util, Context,
};
use ambient_project::{Component, ComponentType, IdentifierPath, IdentifierPathBuf};
use proc_macro2::TokenStream;
use quote::quote;
use thiserror::Error;

pub fn tree_to_token_stream(
    tree: &Tree<Component>,
    context: &Context,
    project_path: IdentifierPath,
) -> anyhow::Result<proc_macro2::TokenStream> {
    to_token_stream(
        tree.root(),
        context,
        &match context {
            Context::Host => quote! {},
            Context::Guest { api_path, .. } => quote! {
                use #api_path::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
            },
        },
        project_path,
    )
}

fn to_token_stream(
    node: &TreeNode<Component>,
    context: &Context,
    prelude: &TokenStream,
    project_path: IdentifierPath,
) -> anyhow::Result<TokenStream> {
    util::tree_to_token_stream(
        node,
        context,
        prelude,
        |node, context, prelude| to_token_stream(node, context, prelude, project_path),
        |name, component, context| {
            let name_ident: syn::Path = syn::parse_str(name)?;
            let name_uppercase_ident: syn::Path = syn::parse_str(&name.to_ascii_uppercase())?;
            let component_ty = type_to_token_stream(&component.type_, context, false)?;

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

#[derive(Error, Debug)]
pub enum TypeTokenStreamError {
    #[error("invalid primitive type")]
    InvalidPrimitiveType,
    #[error("invalid container type")]
    InvalidContainerType,
    #[error("invalid element type")]
    InvalidElementType,
}

pub fn type_to_token_stream(
    ty: &ComponentType,
    context: &Context,
    with_turbofish: bool,
) -> Result<proc_macro2::TokenStream, TypeTokenStreamError> {
    match ty {
        ComponentType::String(ty) => convert_primitive_type_to_rust_type(ty, context)
            .ok_or(TypeTokenStreamError::InvalidPrimitiveType),
        ComponentType::ContainerType {
            type_,
            element_type,
        } => {
            if let Some(element_type) = element_type {
                let container_ty = convert_container_type_to_rust_type(type_)
                    .ok_or(TypeTokenStreamError::InvalidContainerType)?;

                let element_ty = convert_primitive_type_to_rust_type(element_type, context)
                    .ok_or(TypeTokenStreamError::InvalidElementType)?;

                if with_turbofish {
                    Ok(quote! { #container_ty :: < #element_ty > })
                } else {
                    Ok(quote! { #container_ty < #element_ty > })
                }
            } else {
                Ok(convert_primitive_type_to_rust_type(type_, context)
                    .ok_or(TypeTokenStreamError::InvalidPrimitiveType)?)
            }
        }
    }
}

fn convert_primitive_type_to_rust_type(
    ty: &str,
    context: &Context,
) -> Option<proc_macro2::TokenStream> {
    let fully_qualified_prefix = match context {
        Context::Host => quote! {},
        Context::Guest {
            api_path,
            fully_qualified_path,
        } => {
            if *fully_qualified_path {
                quote! { #api_path::global:: }
            } else {
                quote! {}
            }
        }
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
        "U8" => Some(quote! {u8}),
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
