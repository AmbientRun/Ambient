use super::identifier::{Identifier, IdentifierPathBuf};
use anyhow::Context;
use quote::quote;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize, Debug, Clone)]
pub struct Manifest {
    pub project: Project,
    #[serde(default)]
    pub components: BTreeMap<IdentifierPathBuf, NamespaceOrComponent>,
    #[serde(default)]
    pub concepts: BTreeMap<IdentifierPathBuf, NamespaceOrConcept>,
}
impl Manifest {
    pub fn project_path(&self) -> IdentifierPathBuf {
        self.project
            .organization
            .iter()
            .chain(std::iter::once(&self.project.id))
            .cloned()
            .collect()
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Project {
    pub id: Identifier,
    pub organization: Option<Identifier>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Namespace {
    pub name: String,
    pub description: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum NamespaceOr<T> {
    Other(T),
    Namespace(Namespace),
}

pub type NamespaceOrComponent = NamespaceOr<Component>;
pub type NamespaceOrConcept = NamespaceOr<Concept>;
impl<T> From<Namespace> for NamespaceOr<T> {
    fn from(value: Namespace) -> Self {
        Self::Namespace(value)
    }
}
impl From<Component> for NamespaceOr<Component> {
    fn from(value: Component) -> Self {
        Self::Other(value)
    }
}
impl From<Concept> for NamespaceOr<Concept> {
    fn from(value: Concept) -> Self {
        Self::Other(value)
    }
}

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

#[derive(Deserialize, Debug, Clone)]
pub struct Concept {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub extends: Vec<IdentifierPathBuf>,
    pub components: BTreeMap<IdentifierPathBuf, toml::Value>,
}
