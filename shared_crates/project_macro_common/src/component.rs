use super::{
    tree::{Tree, TreeNode, TreeNodeInner, TreeNodeNamespace},
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
    let tree_output = to_token_stream(
        tree.root(),
        context,
        |context, ns, ts| match context {
            Context::Host => {
                let namespace_path = IdentifierPath(ns.path.split_first().unwrap().1).to_string();
                quote! {
                    use glam::{Vec2, Vec3, Vec4, UVec2, UVec3, UVec4, Mat4, Quat};
                    use ambient_ecs::{EntityId, Debuggable, Networked, Store, Resource, Name, Description};
                    ambient_ecs::components!(#namespace_path, {
                        #ts
                    });
                }
            }
            Context::Guest { api_path, .. } => quote! {
                use #api_path::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
                #ts
            },
        },
        project_path,
    )?;

    let init_all_components = {
        fn get_namespaces<'a>(
            ns: &'a TreeNodeNamespace<Component>,
            path: IdentifierPath<'a>,
        ) -> Vec<IdentifierPath<'a>> {
            let mut result = vec![];
            if ns
                .children
                .iter()
                .any(|child| matches!(child.1.inner, TreeNodeInner::Other(_)))
            {
                result.push(path);
            }
            for child in &ns.children {
                match &child.1.inner {
                    TreeNodeInner::Namespace(ns) => {
                        result.append(&mut get_namespaces(ns, child.1.path.as_path()));
                    }
                    _ => {}
                }
            }
            result
        }

        let namespaces = get_namespaces(tree.root_namespace(), IdentifierPath(&[]));

        quote! {
            fn init() {
                #(
                    #namespaces::init_components();
                )*
            }
        }
    };

    Ok(quote! {
        #tree_output
        #init_all_components
    })
}

fn to_token_stream(
    node: &TreeNode<Component>,
    context: &Context,
    wrapper: impl Fn(&Context, &TreeNode<Component>, TokenStream) -> TokenStream + Copy,
    project_path: IdentifierPath,
) -> anyhow::Result<TokenStream> {
    util::tree_to_token_stream(
        node,
        context,
        wrapper,
        |node, context, wrapper| to_token_stream(node, context, wrapper, project_path),
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
                doc_comment += &format!("\n\n*Attributes*: {}", component.attributes.join(", "))
            }
            if let Some(default) = component.default.as_ref() {
                doc_comment += &format!("\n\n*Suggested Default*: {default}")
            }

            let id =
                IdentifierPathBuf::from_iter(project_path.iter().chain(node.path.iter()).cloned())
                    .to_string();
            let doc_comment = doc_comment.trim();

            match context {
                Context::Host => {
                    let attrs = component
                        .attributes
                        .iter()
                        .map(|a| syn::Ident::new(a, proc_macro2::Span::call_site()));

                    let description = &component.description;

                    Ok(quote! {
                        #[doc = #doc_comment]
                        @[#(#attrs,)* Name[#name], Description[#description]]
                        #name_ident: #component_ty,
                    })
                }
                Context::Guest { .. } => Ok(quote! {
                    static #name_uppercase_ident: Lazy< Component< #component_ty > > = Lazy::new(|| __internal_get_component(#id));
                    #[doc = #doc_comment]
                    pub fn #name_ident() -> Component< #component_ty > { *#name_uppercase_ident }
                }),
            }
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
