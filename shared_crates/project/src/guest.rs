use crate::{ComponentType, Identifier, IdentifierPath, IdentifierPathBuf};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use thiserror::Error;

impl<'a> ToTokens for IdentifierPath<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_separated(self.0.iter(), quote::quote! {::})
    }
}

impl ToTokens for IdentifierPathBuf {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.as_path().to_tokens(tokens)
    }
}
impl ToTokens for Identifier {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(Ident::new(self.as_ref(), Span::call_site()))
    }
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
