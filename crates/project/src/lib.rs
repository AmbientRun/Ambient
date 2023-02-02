use std::collections::HashMap;

use elements_ecs::{components, Networked, Store};
use serde::Deserialize;

components!("project", {
    @[Networked, Store]
    description: String,
});

#[derive(Deserialize, Clone, Debug)]
pub struct Manifest {
    pub project: Project,
    #[serde(default)]
    pub components: HashMap<String, Component>,
}
impl Manifest {
    pub fn from_str(manifest: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(manifest)
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct Project {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    pub organization: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Component {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub type_: ComponentType,
}

#[derive(Deserialize, Clone, Debug)]
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
