use std::{collections::BTreeMap, path::PathBuf};

use anyhow::Context;
use quote::quote;
use serde::Deserialize;

#[cfg(test)]
mod tests;

pub fn read_file(file_path: String) -> anyhow::Result<(String, String)> {
    let file_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").context("no manifest dir")?)
        .join(file_path);
    let file_path_str = format!("{}", file_path.display());

    let contents = std::fs::read_to_string(&file_path)?;

    Ok((file_path_str, contents))
}

pub fn implementation(
    (file_path, contents): (String, String),
    extend_paths: &[Vec<String>],
    global_namespace: bool,
) -> anyhow::Result<proc_macro2::TokenStream> {
    let manifest: Manifest = toml::from_str(&contents)?;

    #[derive(Debug)]
    enum TreeNodeInner {
        Module(BTreeMap<String, TreeNode>),
        Component(Component),
        UseAll(Vec<String>),
    }

    #[derive(Debug)]
    struct TreeNode {
        path: Vec<String>,
        inner: TreeNodeInner,
    }
    impl TreeNode {
        fn new(path: Vec<String>, inner: TreeNodeInner) -> Self {
            Self { path, inner }
        }
    }

    let mut root = BTreeMap::new();
    fn insert_into_root(
        root: &mut BTreeMap<String, TreeNode>,
        segments: &[String],
        inner: TreeNodeInner,
    ) -> anyhow::Result<()> {
        let mut manifest_head = root;
        let (leaf_id, modules) = segments.split_last().context("empty segments")?;

        let mut segments_so_far = vec![];
        for segment in modules {
            segments_so_far.push(segment.to_string());

            let new_head = manifest_head
                .entry(segment.to_string())
                .or_insert(TreeNode::new(
                    segments_so_far.clone(),
                    TreeNodeInner::Module(Default::default()),
                ));

            manifest_head = match &mut new_head.inner {
                TreeNodeInner::Module(module) => module,
                _ => anyhow::bail!("found a non-module where a module was expected"),
            };
        }

        manifest_head.insert(
            leaf_id.clone(),
            TreeNode::new(segments.iter().map(|s| s.to_string()).collect(), inner),
        );

        Ok(())
    }
    for (id, component) in manifest.components {
        insert_into_root(
            &mut root,
            &id.split("::").map(|s| s.to_string()).collect::<Vec<_>>(),
            TreeNodeInner::Component(component),
        )?;
    }
    for path in extend_paths {
        let components_index = path
            .iter()
            .position(|s| s == "components")
            .context("expected components:: in extend path")?;
        let mut subpath = path[components_index + 1..path.len()].to_vec();
        subpath.push("#use_all#".to_string());

        insert_into_root(&mut root, &subpath, TreeNodeInner::UseAll(path.clone()))?;
    }

    fn expand_tree(
        tree_node: &TreeNode,
        project_path: &[String],
    ) -> anyhow::Result<proc_macro2::TokenStream> {
        let name = tree_node
            .path
            .last()
            .map(|s| s.as_str())
            .unwrap_or_default();
        match &tree_node.inner {
            TreeNodeInner::Module(module) => {
                let children = module
                    .values()
                    .map(|child| expand_tree(child, project_path))
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(if name.is_empty() {
                    quote! {
                        #(#children)*
                    }
                } else {
                    let name_ident: syn::Ident = syn::parse_str(name)?;
                    quote! {
                        pub mod #name_ident {
                            #(#children)*
                        }
                    }
                })
            }
            TreeNodeInner::Component(component) => {
                let name_ident: syn::Ident = syn::parse_str(name)?;
                let name_uppercase_ident: syn::Ident = syn::parse_str(&name.to_ascii_uppercase())?;
                let component_ty = component.type_.to_token_stream()?;
                let attributes_str = if component.attributes.is_empty() {
                    String::new()
                } else {
                    format!("*Attributes*: {}", component.attributes.clone().join(", "))
                };

                let doc_comment = [
                    format!("**{}**", component.name),
                    component.description.clone(),
                    attributes_str,
                ]
                .into_iter()
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("\n\n");

                let id = [project_path, &tree_node.path].concat().join("::");

                Ok(quote! {
                    static #name_uppercase_ident: crate::LazyComponent<#component_ty> = crate::lazy_component!(#id);
                    #[doc = #doc_comment]
                    pub fn #name_ident() -> crate::Component<#component_ty> { *#name_uppercase_ident }
                })
            }
            TreeNodeInner::UseAll(path) => {
                let path = path
                    .iter()
                    .map(|s| syn::parse_str::<syn::Ident>(s))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(quote! {
                    pub use #(#path::)* *;
                })
            }
        }
    }

    let project_path: Vec<_> = if global_namespace {
        vec![]
    } else {
        manifest
            .project
            .organization
            .iter()
            .chain(std::iter::once(&manifest.project.id))
            .cloned()
            .collect()
    };
    let expanded_tree = expand_tree(
        &TreeNode::new(vec![], TreeNodeInner::Module(root)),
        &project_path,
    )?;
    Ok(quote!(
        const _PROJECT_MANIFEST: &'static str = include_str!(#file_path);
        #[allow(missing_docs)]
        pub mod components {
            #expanded_tree
        }
    ))
}

#[derive(Deserialize, Debug)]
struct Manifest {
    project: Project,
    #[serde(default)]
    components: BTreeMap<String, Component>,
}

#[derive(Deserialize, Debug)]
pub struct Project {
    id: String,
    organization: Option<String>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Component {
    name: String,
    description: String,
    #[serde(rename = "type")]
    type_: ComponentType,
    #[serde(default)]
    attributes: Vec<String>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum ComponentType {
    String(String),
    ContainerType {
        #[serde(rename = "type")]
        type_: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        element_type: Option<String>,
    },
}
impl ComponentType {
    fn convert_primitive_type_to_rust_type(ty: &str) -> Option<proc_macro2::TokenStream> {
        match ty {
            "Empty" => Some(quote! {()}),
            "Bool" => Some(quote! {bool}),
            "EntityId" => Some(quote! {crate::EntityId}),
            "F32" => Some(quote! {f32}),
            "F64" => Some(quote! {f64}),
            "Mat4" => Some(quote! {crate::Mat4}),
            "I32" => Some(quote! {i32}),
            "Quat" => Some(quote! {crate::Quat}),
            "String" => Some(quote! {String}),
            "U32" => Some(quote! {u32}),
            "U64" => Some(quote! {u64}),
            "Vec2" => Some(quote! {crate::Vec2}),
            "Vec3" => Some(quote! {crate::Vec3}),
            "Vec4" => Some(quote! {crate::Vec4}),
            "ObjectRef" => Some(quote! {crate::ObjectRef}),
            "EntityUid" => Some(quote! {crate::EntityUid}),
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

    fn to_token_stream(&self) -> anyhow::Result<proc_macro2::TokenStream> {
        match self {
            ComponentType::String(ty) => {
                Self::convert_primitive_type_to_rust_type(ty).context("invalid primitive type")
            }
            ComponentType::ContainerType {
                type_,
                element_type,
            } => {
                let container_ty = Self::convert_container_type_to_rust_type(type_)
                    .context("invalid container type")?;
                let element_ty = element_type
                    .as_deref()
                    .map(Self::convert_primitive_type_to_rust_type)
                    .context("invalid element type")?;
                Ok(if let Some(element_ty) = element_ty {
                    quote! { #container_ty < #element_ty > }
                } else {
                    container_ty
                })
            }
        }
    }
}
