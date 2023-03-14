use std::{collections::HashMap, fmt::Display};

use ambient_ecs::{
    components, Debuggable, ExternalComponentAttributes, ExternalComponentDesc, ExternalComponentFlagAttributes, Networked,
    PrimitiveComponentType, Store,
};
use serde::{de::Visitor, Deserialize, Serialize};
use thiserror::Error;

#[cfg(test)]
mod tests;

components!("project", {
    @[Networked, Store, Debuggable]
    description: String,
});

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Manifest {
    pub project: Project,
    #[serde(default)]
    pub build: Build,
    #[serde(default)]
    pub components: HashMap<IdentifierPathBuf, NamespaceOrComponent>,
    #[serde(default)]
    pub concepts: HashMap<IdentifierPathBuf, NamespaceOrConcept>,
}
impl Manifest {
    pub fn parse(manifest: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(manifest)
    }

    pub fn all_defined_components(&self, global_namespace: bool) -> Result<Vec<ExternalComponentDesc>, &'static str> {
        let project_path: Vec<_> = if global_namespace {
            vec![]
        } else {
            self.project.organization.iter().chain(std::iter::once(&self.project.id)).cloned().collect()
        };

        self.components
            .iter()
            .filter_map(|(id, component)| match component {
                NamespaceOrComponent::Other(c) => Some((id, c)),
                NamespaceOrComponent::Namespace(_) => None,
            })
            .map(|(id, component)| {
                let full_path = IdentifierPathBuf(project_path.iter().chain(id.0.iter()).cloned().collect());
                Ok(ExternalComponentDesc {
                    path: full_path.to_string(),
                    ty: (&component.type_).try_into()?,
                    attributes: ExternalComponentAttributes {
                        name: Some(component.name.clone()),
                        description: Some(component.description.clone()),
                        flags: ExternalComponentFlagAttributes::from_iter(component.attributes.iter().map(|s| s.as_str())),
                    },
                })
            })
            .collect::<Result<Vec<_>, _>>()
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Project {
    pub id: Identifier,
    pub name: Option<String>,
    pub version: Version,
    pub description: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    pub organization: Option<Identifier>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Build {
    #[serde(default)]
    pub rust: BuildRust,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct BuildRust {
    #[serde(rename = "feature-multibuild")]
    pub feature_multibuild: Vec<String>,
}
impl Default for BuildRust {
    fn default() -> Self {
        Self { feature_multibuild: vec!["server".to_string()] }
    }
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

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Component {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub type_: ComponentType,
    #[serde(default)]
    pub attributes: Vec<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
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
impl TryFrom<&ComponentType> for PrimitiveComponentType {
    type Error = &'static str;

    fn try_from(value: &ComponentType) -> Result<Self, Self::Error> {
        match value {
            ComponentType::String(ty) => PrimitiveComponentType::try_from(ty.as_str()),
            ComponentType::ContainerType { type_, element_type } => {
                let element_ty = element_type.as_deref().map(PrimitiveComponentType::try_from).transpose()?;
                match element_ty {
                    Some(element_ty) => match type_.as_str() {
                        "Vec" => element_ty.to_vec_type().ok_or("invalid element type for Vec"),
                        "Option" => element_ty.to_option_type().ok_or("invalid element type for Option"),
                        _ => Err("invalid container type"),
                    },
                    None => PrimitiveComponentType::try_from(type_.as_str()),
                }
            }
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Concept {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub extends: Vec<IdentifierPathBuf>,
    pub components: HashMap<IdentifierPathBuf, toml::Value>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IdentifierPathBuf(Vec<Identifier>);
impl IdentifierPathBuf {
    pub fn new(path: impl Into<String>) -> Result<Self, &'static str> {
        Self::new_impl(path.into())
    }

    fn new_impl(path: String) -> Result<Self, &'static str> {
        Ok(Self(path.split("::").map(Identifier::new).collect::<Result<_, _>>()?))
    }
}
impl Display for IdentifierPathBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for id in &self.0 {
            if !first {
                write!(f, "::")?;
            }

            write!(f, "{}", id.0)?;
            first = false;
        }

        Ok(())
    }
}
impl Serialize for IdentifierPathBuf {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        String::serialize(&self.to_string(), serializer)
    }
}
impl<'de> Deserialize<'de> for IdentifierPathBuf {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        IdentifierPathBuf::new_impl(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identifier(String);
impl Identifier {
    pub fn new(id: impl Into<String>) -> Result<Self, &'static str> {
        Self::new_impl(id.into())
    }

    fn new_impl(id: String) -> Result<Self, &'static str> {
        Self::validate(&id)?;
        Ok(Self(id))
    }

    pub fn validate(id: &str) -> Result<&str, &'static str> {
        if id.is_empty() {
            return Err("identifier must not be empty");
        }

        if !id.starts_with(|c: char| c.is_ascii_lowercase()) {
            return Err("identifier must start with a lowercase ASCII character");
        }

        if !id.chars().all(|c| c.is_ascii_lowercase() || c.is_numeric() || c == '_') {
            return Err("identifier must be snake-case ASCII");
        }

        Ok(id)
    }
}
impl Serialize for Identifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        String::serialize(&self.0, serializer)
    }
}
impl<'de> Deserialize<'de> for Identifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Identifier::new_impl(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}
impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}
impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    pub fn new_from_str(id: &str) -> Result<Self, VersionError> {
        if id.trim().is_empty() {
            return Err(VersionError::TooFewComponents);
        }

        let mut segments = id.split('.');
        let major = segments.next().ok_or(VersionError::TooFewComponents)?.parse()?;
        let minor = segments.next().map(|s| s.parse()).transpose()?.unwrap_or(0);
        let patch = segments.next().map(|s| s.parse()).transpose()?.unwrap_or(0);

        if segments.next().is_some() {
            return Err(VersionError::TooManyComponents);
        }

        if [major, minor, patch].iter().all(|v| *v == 0) {
            return Err(VersionError::AllZero);
        }

        Ok(Self { major, minor, patch })
    }
}
impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}
impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(VersionVisitor)
    }
}
struct VersionVisitor;
impl<'de> Visitor<'de> for VersionVisitor {
    type Value = Version;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a semantic dot-separated version with up to three components")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Version::new_from_str(v).map_err(serde::de::Error::custom)
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum VersionError {
    #[error("invalid number in version segment")]
    InvalidNumber(#[from] std::num::ParseIntError),
    #[error("too few components in version (at least one required)")]
    TooFewComponents,
    #[error("too many components (at most three required)")]
    TooManyComponents,
    #[error("all components were zero")]
    AllZero,
}
