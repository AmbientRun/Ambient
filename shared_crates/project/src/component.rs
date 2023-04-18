use serde::Deserialize;

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Component {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub type_: ComponentType,
    #[serde(default)]
    pub attributes: Vec<String>,
    #[serde(default)]
    pub default: Option<toml::Value>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum ComponentType {
    String(String),
    ContainerType {
        #[serde(rename = "type")]
        #[serde(alias = "container_type")]
        type_: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        element_type: Option<String>,
    },
}
