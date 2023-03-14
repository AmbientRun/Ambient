use super::{
    component::Component,
    concept::Concept,
    identifier::{Identifier, IdentifierPathBuf},
};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize, Debug, Clone)]
pub struct Manifest {
    pub project: Project,
    #[serde(default)]
    pub components: BTreeMap<IdentifierPathBuf, NamespaceOr<Component>>,
    #[serde(default)]
    pub concepts: BTreeMap<IdentifierPathBuf, NamespaceOr<Concept>>,
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
