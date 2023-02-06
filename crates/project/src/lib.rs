use std::{collections::HashMap, fmt::Display};

use elements_ecs::{components, Networked, PrimitiveComponentType, Store};
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

components!("project", {
    @[Networked, Store]
    description: String,
});

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Manifest {
    pub project: Project,
    #[serde(default)]
    pub components: HashMap<IdentifierPath, Component>,
}
impl Manifest {
    pub fn parse(manifest: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(manifest)
    }

    pub fn all_defined_components(&self, global_namespace: bool) -> Result<Vec<(String, PrimitiveComponentType)>, &'static str> {
        let project_path: Vec<_> = if global_namespace {
            vec![]
        } else {
            self.project.organization.iter().chain(std::iter::once(&self.project.name)).cloned().collect()
        };

        self.components
            .iter()
            .map(|(id, component)| {
                let full_path = IdentifierPath(project_path.iter().chain(id.0.iter()).cloned().collect());
                Ok((full_path.to_string(), (&component.type_).try_into()?))
            })
            .collect::<Result<Vec<_>, _>>()
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Project {
    pub name: Identifier,
    pub version: String,
    pub description: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    pub organization: Option<Identifier>,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Component {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub type_: ComponentType,
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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IdentifierPath(Vec<Identifier>);
impl IdentifierPath {
    pub fn new(path: impl Into<String>) -> Result<Self, &'static str> {
        Self::new_impl(path.into())
    }

    fn new_impl(path: String) -> Result<Self, &'static str> {
        Ok(Self(path.split("::").map(|s| Identifier::new(s)).collect::<Result<_, _>>()?))
    }
}
impl Display for IdentifierPath {
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
impl Serialize for IdentifierPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        String::serialize(&self.to_string(), serializer)
    }
}
impl<'de> Deserialize<'de> for IdentifierPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        IdentifierPath::new_impl(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
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
