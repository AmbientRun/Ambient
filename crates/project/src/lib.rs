use std::collections::HashMap;

use elements_ecs::{components, Networked, PrimitiveComponentType, Store};
use serde::Deserialize;

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
    pub components: HashMap<String, Component>,
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
                let full_path = project_path.iter().map(|s| s.as_str()).chain(id.split("::")).collect::<Vec<_>>();
                Ok((full_path.join("::"), (&component.type_).try_into()?))
            })
            .collect::<Result<Vec<_>, _>>()
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Project {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    pub organization: Option<String>,
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
