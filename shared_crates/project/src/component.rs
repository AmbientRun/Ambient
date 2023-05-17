use serde::{Deserialize, Serialize};

use crate::CamelCaseIdentifier;

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
pub struct Component {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub type_: ComponentType,
    #[serde(default)]
    pub attributes: Vec<CamelCaseIdentifier>,
    #[serde(default)]
    pub default: Option<toml::Value>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum ComponentType {
    Identifier(CamelCaseIdentifier),
    ContainerType {
        #[serde(rename = "type")]
        #[serde(alias = "container_type")]
        type_: CamelCaseIdentifier,
        element_type: CamelCaseIdentifier,
    },
}
