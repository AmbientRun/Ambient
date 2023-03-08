use super::{
    identifier::{Identifier, IdentifierPath, IdentifierPathBuf},
    manifest::{Namespace, NamespaceOr},
};
use anyhow::Context;
use std::{collections::BTreeMap, fmt::Debug};

#[derive(Debug, Clone)]
pub struct Tree<T: Clone + Debug> {
    root: TreeNode<T>,
}
impl<T: Clone + Debug> Tree<T> {
    pub(super) fn new(
        values: &BTreeMap<IdentifierPathBuf, NamespaceOr<T>>,
        validate_namespaces_documented: bool,
    ) -> anyhow::Result<Self> {
        let mut tree = Self {
            root: TreeNode::new(
                IdentifierPathBuf::empty(),
                TreeNodeInner::Namespace(TreeNodeNamespace {
                    children: BTreeMap::new(),
                    namespace: None,
                }),
            ),
        };

        for (id, namespace_or_other) in values {
            let node = match namespace_or_other {
                NamespaceOr::Namespace(n) => {
                    TreeNodeInner::Namespace(TreeNodeNamespace::new(Some(n.clone())))
                }
                NamespaceOr::Other(v) => TreeNodeInner::Other(v.clone()),
            };

            tree.insert(id.clone(), node)?;
        }

        if validate_namespaces_documented {
            for node in tree.root_namespace().children.values() {
                ensure_namespace_documented(node)?;
            }
        }

        Ok(tree)
    }

    pub fn get(&self, path: IdentifierPath) -> Option<&T> {
        self.root_namespace().get(path)
    }

    pub fn root(&self) -> &TreeNode<T> {
        &self.root
    }

    pub fn root_namespace(&self) -> &TreeNodeNamespace<T> {
        match &self.root.inner {
            TreeNodeInner::Namespace(namespace) => namespace,
            _ => unreachable!(),
        }
    }

    fn root_namespace_mut(&mut self) -> &mut TreeNodeNamespace<T> {
        match &mut self.root.inner {
            TreeNodeInner::Namespace(namespace) => namespace,
            _ => unreachable!(),
        }
    }

    fn insert(&mut self, path: IdentifierPathBuf, inner: TreeNodeInner<T>) -> anyhow::Result<()> {
        let mut manifest_head = &mut self.root_namespace_mut().children;
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
pub struct TreeNode<T: Clone + Debug> {
    pub path: IdentifierPathBuf,
    pub inner: TreeNodeInner<T>,
}
impl<T: Clone + Debug> TreeNode<T> {
    pub fn new(path: IdentifierPathBuf, inner: TreeNodeInner<T>) -> Self {
        Self { path, inner }
    }
}

#[derive(Debug, Clone)]
pub enum TreeNodeInner<T: Clone + Debug> {
    Namespace(TreeNodeNamespace<T>),
    Other(T),
}

#[derive(Debug, Clone)]
pub struct TreeNodeNamespace<T: Clone + Debug> {
    pub children: BTreeMap<Identifier, TreeNode<T>>,
    pub namespace: Option<Namespace>,
}
impl<T: Clone + Debug> TreeNodeNamespace<T> {
    fn new(namespace: Option<Namespace>) -> Self {
        Self {
            children: BTreeMap::new(),
            namespace,
        }
    }

    fn get(&self, path: IdentifierPath) -> Option<&T> {
        let (root, rest) = path.split_first()?;
        let child = self.children.get(root)?;
        match &child.inner {
            TreeNodeInner::Namespace(ns) => ns.get(IdentifierPath(rest)),
            TreeNodeInner::Other(v) => Some(v),
        }
    }
}

fn ensure_namespace_documented<T: Clone + Debug>(node: &TreeNode<T>) -> anyhow::Result<()> {
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
