use quote::quote;
use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize, Clone, Debug, PartialEq)]
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

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum ComponentType {
    String(String),
    ContainerType {
        #[serde(rename = "type")]
        #[serde(alias = "container_type")]
        type_: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        element_type: Option<String>,
    },
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

impl ComponentType {
    pub fn to_token_stream(
        &self,
        api_name: &syn::Path,
        fully_qualified: bool,
        with_turbofish: bool,
    ) -> Result<proc_macro2::TokenStream, TypeTokenStreamError> {
        match self {
            ComponentType::String(ty) => {
                convert_primitive_type_to_rust_type(ty, api_name, fully_qualified)
                    .ok_or(TypeTokenStreamError::InvalidPrimitiveType)
            }
            ComponentType::ContainerType {
                type_,
                element_type,
            } => {
                if let Some(element_type) = element_type {
                    let container_ty = convert_container_type_to_rust_type(type_)
                        .ok_or(TypeTokenStreamError::InvalidContainerType)?;

                    let element_ty = convert_primitive_type_to_rust_type(
                        element_type,
                        api_name,
                        fully_qualified,
                    )
                    .ok_or(TypeTokenStreamError::InvalidElementType)?;

                    if with_turbofish {
                        Ok(quote! { #container_ty :: < #element_ty > })
                    } else {
                        Ok(quote! { #container_ty < #element_ty > })
                    }
                } else {
                    Ok(
                        convert_primitive_type_to_rust_type(type_, api_name, fully_qualified)
                            .ok_or(TypeTokenStreamError::InvalidPrimitiveType)?,
                    )
                }
            }
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
