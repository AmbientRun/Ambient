use std::collections::BTreeMap;

use super::{
    components::Tree as ComponentTree,
    identifier::{Identifier, IdentifierPath, IdentifierPathBuf},
    manifest::{ComponentType, Concept, Manifest, Namespace, NamespaceOrOther},
};
use anyhow::Context;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub struct Tree {
    root: TreeNodeNamespace,
}
impl Tree {
    pub(super) fn new(
        manifest: &Manifest,
        validate_namespaces_documented: bool,
    ) -> anyhow::Result<Self> {
        let mut tree = Self {
            root: TreeNodeNamespace {
                children: BTreeMap::new(),
                namespace: None,
            },
        };

        for (id, namespace_or_other) in manifest.concepts.clone() {
            let node = match namespace_or_other {
                NamespaceOrOther::Namespace(n) => {
                    TreeNodeInner::Namespace(TreeNodeNamespace::new(Some(n)))
                }
                NamespaceOrOther::Concept(c) => TreeNodeInner::Other(c),
                _ => unreachable!(),
            };

            tree.insert(id, node)?;
        }

        if validate_namespaces_documented {
            for node in tree.root.children.values() {
                ensure_namespace_documented(node)?;
            }
        }

        Ok(tree)
    }

    pub fn to_token_stream(
        &self,
        components_tree: &ComponentTree,
        api_name: &syn::Path,
    ) -> anyhow::Result<proc_macro2::TokenStream> {
        TreeNode::new(
            IdentifierPathBuf::empty(),
            TreeNodeInner::Namespace(self.root.clone()),
        )
        .to_token_stream(components_tree, api_name)
    }

    fn insert(
        &mut self,
        path: IdentifierPathBuf,
        inner: TreeNodeInner<Concept>,
    ) -> anyhow::Result<()> {
        let mut manifest_head = &mut self.root.children;
        let (leaf_id, namespaces) = path.split_last().context("empty segments")?;

        let mut segments_so_far = IdentifierPathBuf::empty();
        for segment in namespaces {
            segments_so_far.push(segment.clone());

            let new_head = manifest_head
                .entry(segment.clone())
                .or_insert(TreeNode::new(
                    segments_so_far.clone(),
                    TreeNodeInner::Namespace(TreeNodeNamespace::new(None)),
                ));

            manifest_head = match &mut new_head.inner {
                TreeNodeInner::Namespace(ns) => &mut ns.children,
                _ => anyhow::bail!("found a non-namespace where a namespace was expected"),
            };
        }

        match manifest_head.get_mut(leaf_id) {
            Some(leaf) => {
                leaf.inner = match (leaf.inner.clone(), inner.clone()) {
                    (
                        TreeNodeInner::Namespace(TreeNodeNamespace {
                            children: mut existing,
                            namespace: None,
                        }),
                        TreeNodeInner::Namespace(TreeNodeNamespace {
                            children: mut new,
                            namespace: Some(ns),
                        }),
                    ) => {
                        existing.append(&mut new);
                        TreeNodeInner::Namespace(TreeNodeNamespace {
                            children: existing,
                            namespace: Some(ns),
                        })
                    }
                    _ => anyhow::bail!(
                        "Attempted to replace {:?} at `{}` with {:?}",
                        leaf.inner,
                        path,
                        inner
                    ),
                };
            }
            None => {
                manifest_head.insert(leaf_id.clone(), TreeNode::new(path, inner));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
struct TreeNode {
    path: IdentifierPathBuf,
    inner: TreeNodeInner<Concept>,
}
impl TreeNode {
    fn new(path: IdentifierPathBuf, inner: TreeNodeInner<Concept>) -> Self {
        Self { path, inner }
    }

    fn to_token_stream(
        &self,
        components_tree: &ComponentTree,
        api_name: &syn::Path,
    ) -> anyhow::Result<proc_macro2::TokenStream> {
        let name = self.path.last().map(|s| s.as_ref()).unwrap_or_default();
        match &self.inner {
            TreeNodeInner::Namespace(ns) => {
                let children = ns
                    .children
                    .values()
                    .map(|child| child.to_token_stream(components_tree, api_name))
                    .collect::<Result<Vec<_>, _>>()?;

                let prelude = quote! {
                    use super::components;
                    use #api_name::prelude::*;
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
            TreeNodeInner::Other(concept) => {
                let make_concept = self.generate_make(components_tree, name, concept)?;
                let is_concept = self.generate_is(name, concept)?;
                Ok(quote! {
                    #make_concept
                    #is_concept
                })
            }
        }
    }

    fn generate_make(
        &self,
        components_tree: &ComponentTree,
        name: &str,
        concept: &Concept,
    ) -> anyhow::Result<TokenStream> {
        let make_comment = format!("Makes a {} ({})", concept.name, concept.description);
        let make_ident = quote::format_ident!("make_{}", name);

        let extends: Vec<_> = concept
            .extends
            .iter()
            .map(|i| {
                let extend_ident = quote::format_ident!("make_{}", i.as_ref());
                quote! {
                    with_merge(#extend_ident())
                }
            })
            .collect();

        let components_prefix = Identifier::new("components").map_err(anyhow::Error::msg)?;
        let components = concept
            .components
            .iter()
            .map(|component| {
                let full_path = build_component_path(&components_prefix, component.0.as_path());

                let manifest_component =
                    components_tree
                        .get(component.0.as_path())
                        .with_context(|| {
                            format!("there is no component defined at `{}`", component.0)
                        })?;

                let default = toml_value_to_tokens(
                    component.0.as_path(),
                    &manifest_component.type_,
                    component.1,
                )?;

                Ok(quote! { with(#full_path(), #default) })
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        Ok(quote! {
            #[allow(clippy::approx_constant)]
            #[doc = #make_comment]
            pub fn #make_ident() -> Entity {
                Entity::new()
                    #(.#extends)*
                    #(.#components)*
            }
        })
    }

    fn generate_is(&self, name: &str, concept: &Concept) -> anyhow::Result<TokenStream> {
        let is_comment = format!(
            "Checks if the entity is a {} ({})",
            concept.name, concept.description
        );
        let is_ident = quote::format_ident!("is_{}", name);

        let extends: Vec<_> = concept
            .extends
            .iter()
            .map(|i| {
                let extend_ident = quote::format_ident!("is_{}", i.as_ref());
                quote! {
                    #extend_ident(id)
                }
            })
            .collect();

        let components_prefix = Identifier::new("components").map_err(anyhow::Error::msg)?;
        let components: Vec<_> = concept
            .components
            .iter()
            .map(|c| build_component_path(&components_prefix, c.0.as_path()))
            .map(|p| quote! { #p() })
            .collect();

        Ok(quote! {
            #[doc = #is_comment]
            pub fn #is_ident(id: EntityId) -> bool {
                #(#extends && )* entity::has_components(id, &[
                    #(&#components),*
                ])
            }
        })
    }
}

#[derive(Debug, Clone)]
enum TreeNodeInner<T> {
    Namespace(TreeNodeNamespace),
    Other(T),
}

#[derive(Debug, Clone)]
struct TreeNodeNamespace {
    pub children: BTreeMap<Identifier, TreeNode>,
    pub namespace: Option<Namespace>,
}
impl TreeNodeNamespace {
    fn new(namespace: Option<Namespace>) -> Self {
        Self {
            children: BTreeMap::new(),
            namespace,
        }
    }
}

fn ensure_namespace_documented(node: &TreeNode) -> anyhow::Result<()> {
    match &node.inner {
        TreeNodeInner::Namespace(TreeNodeNamespace {
            namespace: None, ..
        }) => anyhow::bail!(
            "The namespace `{}` is missing a name and description.",
            node.path
        ),
        TreeNodeInner::Namespace(TreeNodeNamespace {
            children,
            namespace: Some(_),
        }) => {
            for node in children.values() {
                ensure_namespace_documented(node)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn build_component_path(prefix: &Identifier, path: IdentifierPath) -> IdentifierPathBuf {
    IdentifierPathBuf::from_iter(std::iter::once(prefix).chain(path.iter()).cloned())
}

fn toml_value_to_tokens(
    path: IdentifierPath,
    ty: &ComponentType,
    value: &toml::Value,
) -> anyhow::Result<TokenStream> {
    match ty {
        ComponentType::String(ty) => toml_value_to_tokens_primitive(path, ty, value),
        ComponentType::ContainerType {
            type_,
            element_type,
        } => {
            if let Some(element_type) = element_type {
                let values = value.as_array().with_context(|| {
                    format!("expected an array initializer for component `{path}`")
                })?;

                match type_.as_str() {
                    "Vec" => {
                        let values = values
                            .iter()
                            .map(|v| toml_value_to_tokens_primitive(path, element_type, v))
                            .collect::<anyhow::Result<Vec<_>>>()?;

                        Ok(quote! { vec![ #(#values),* ] })
                    }
                    "Option" => {
                        if values.is_empty() {
                            Ok(quote! { None })
                        } else {
                            let value =
                                toml_value_to_tokens_primitive(path, element_type, &values[0])?;
                            Ok(quote! { Some(#value) })
                        }
                    }
                    _ => anyhow::bail!("unsupported container `{type_}` for component `{path}`"),
                }
            } else {
                toml_value_to_tokens_primitive(path, type_, value)
            }
        }
    }
}

fn toml_value_to_tokens_primitive(
    path: IdentifierPath,
    ty: &str,
    value: &toml::Value,
) -> anyhow::Result<TokenStream> {
    Ok(match (ty, value) {
        ("Empty", toml::Value::Table(t)) if t.is_empty() => quote! {()},
        ("Bool", toml::Value::Boolean(b)) => quote! {#b},
        ("EntityId", toml::Value::String(s)) => quote! {EntityId::from_base64(#s)},
        ("F32", toml::Value::Float(f)) => {
            let f = *f as f32;
            quote! {#f}
        }
        ("F64", toml::Value::Float(f)) => {
            quote! {#f}
        }
        ("Mat4", toml::Value::Array(a)) => {
            let arr = toml_array_f32_to_array_tokens(path, a)?;
            quote! { Mat4::from_cols_array(&[#arr]) }
        }
        ("I32", toml::Value::Integer(i)) => {
            let i = *i as i32;
            quote! {#i}
        }
        ("Quat", toml::Value::Array(a)) => {
            let arr = toml_array_f32_to_array_tokens(path, a)?;
            quote! { Quat::from_xyzw(#arr) }
        }
        ("String", toml::Value::String(s)) => quote! {#s.to_string()},
        ("U32", toml::Value::Integer(i)) => {
            let i = *i as u32;
            quote! {#i}
        }
        ("U64", toml::Value::String(s)) => {
            let val: u64 = s.parse()?;
            quote! {#val}
        }
        ("Vec2", toml::Value::Array(a)) => {
            let arr = toml_array_f32_to_array_tokens(path, a)?;
            quote! { Vec2::new(#arr) }
        }
        ("Vec3", toml::Value::Array(a)) => {
            let arr = toml_array_f32_to_array_tokens(path, a)?;
            quote! { Vec3::new(#arr) }
        }
        ("Vec4", toml::Value::Array(a)) => {
            let arr = toml_array_f32_to_array_tokens(path, a)?;
            quote! { Vec4::new(#arr) }
        }
        _ => anyhow::bail!("unsupported type `{ty}` and value `{value}` for component `{path}`"),
    })
}

fn toml_array_f32_to_array_tokens(
    path: IdentifierPath,
    array: &toml::value::Array,
) -> anyhow::Result<TokenStream> {
    let members = array
        .iter()
        .map(|c| {
            if let Some(f) = c.as_float() {
                Ok(f as f32)
            } else if let Some(i) = c.as_integer() {
                Ok(i as f32)
            } else {
                anyhow::bail!(
                    "not all of the values for the array initializer for `{path}` were numbers"
                )
            }
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(quote! { #(#members),* })
}
