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
    pub concepts: BTreeMap<Identifier, Concept>,
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

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum NamespaceOrComponent {
    Component(Component),
    Namespace(Namespace),
}

#[derive(Deserialize, Debug, Clone)]
pub struct Namespace {
    pub name: String,
    pub description: String,
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
    ) -> anyhow::Result<proc_macro2::TokenStream> {
        match self {
            ComponentType::String(ty) => {
                convert_primitive_type_to_rust_type(api_name, ty).context("invalid primitive type")
            }
            ComponentType::ContainerType {
                type_,
                element_type,
            } => {
                if let Some(element_type) = element_type {
                    let container_ty = convert_container_type_to_rust_type(type_)
                        .context("invalid container type")?;

                    let element_ty = convert_primitive_type_to_rust_type(api_name, element_type)
                        .context("invalid element type")?;

                    Ok(quote! { #container_ty < #element_ty > })
                } else {
                    Ok(convert_primitive_type_to_rust_type(api_name, type_)
                        .context("invalid primitive type")?)
                }
            }
        }
    }
}

fn convert_primitive_type_to_rust_type(
    api_name: &syn::Path,
    ty: &str,
) -> Option<proc_macro2::TokenStream> {
    match ty {
        "Empty" => Some(quote! {()}),
        "Bool" => Some(quote! {bool}),
        "EntityId" => Some(quote! {#api_name::global::EntityId}),
        "F32" => Some(quote! {f32}),
        "F64" => Some(quote! {f64}),
        "Mat4" => Some(quote! {#api_name::global::Mat4}),
        "I32" => Some(quote! {i32}),
        "Quat" => Some(quote! {#api_name::global::Quat}),
        "String" => Some(quote! {String}),
        "U32" => Some(quote! {u32}),
        "U64" => Some(quote! {u64}),
        "Vec2" => Some(quote! {#api_name::global::Vec2}),
        "Vec3" => Some(quote! {#api_name::global::Vec3}),
        "Vec4" => Some(quote! {#api_name::global::Vec4}),
        "Uvec2" => Some(quote! {#api_name::global::UVec2}),
        "Uvec3" => Some(quote! {#api_name::global::UVec3}),
        "Uvec4" => Some(quote! {#api_name::global::UVec4}),
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
    pub extends: Vec<Identifier>,
    pub components: BTreeMap<IdentifierPathBuf, toml::Value>,
}
