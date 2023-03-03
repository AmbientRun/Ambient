use super::{
    identifier::{Identifier, IdentifierPath, IdentifierPathBuf},
    manifest::{Component, Manifest, Namespace,  NamespaceOrOther},
};
use anyhow::Context;
use quote::quote;
use std::collections::BTreeMap;

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

        for (id, namespace_or_component) in manifest.components.clone() {
            let node = match namespace_or_component {
                NamespaceOrOther::Namespace(n) => {
                    TreeNodeInner::Namespace(TreeNodeNamespace::new(Some(n)))
                }
                NamespaceOrOther::Component(c) => TreeNodeInner::Other(c),
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
        api_name: &syn::Path,
        project_path: IdentifierPath,
    ) -> anyhow::Result<proc_macro2::TokenStream> {
        TreeNode::new(
            IdentifierPathBuf::empty(),
            TreeNodeInner::Namespace(self.root.clone()),
        )
        .to_token_stream(api_name, project_path)
    }

    pub fn get(&self, path: IdentifierPath) -> Option<&Component> {
        self.root.get(path)
    }

    fn insert(&mut self, path: IdentifierPathBuf, inner: TreeNodeInner<Component>) -> anyhow::Result<()> {
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
    inner: TreeNodeInner<Component>,
}
impl TreeNode {
    fn new(path: IdentifierPathBuf, inner: TreeNodeInner<Component>) -> Self {
        Self { path, inner }
    }

    fn to_token_stream(
        &self,
        api_name: &syn::Path,
        project_path: IdentifierPath,
    ) -> anyhow::Result<proc_macro2::TokenStream> {
        let name = self.path.last().map(|s| s.as_ref()).unwrap_or_default();
        match &self.inner {
            TreeNodeInner::Namespace(ns) => {
                let children = ns
                    .children
                    .values()
                    .map(|child| child.to_token_stream(api_name, project_path))
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

                let id = IdentifierPathBuf::from_iter(
                    project_path.iter().chain(self.path.iter()).cloned(),
                )
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

    fn get(&self, path: IdentifierPath) -> Option<&Component> {
        let (root, rest) = path.split_first()?;
        let child = self.children.get(root)?;
        match &child.inner {
            TreeNodeInner::Namespace(ns) => ns.get(IdentifierPath(rest)),
            TreeNodeInner::Other(c) => Some(c),
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
