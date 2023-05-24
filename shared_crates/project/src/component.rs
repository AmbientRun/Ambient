use serde::{Deserialize, Serialize};

use crate::ItemPathBuf;

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
pub struct Component {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub type_: ComponentType,
    #[serde(default)]
    pub attributes: Vec<ItemPathBuf>,
    #[serde(default)]
    pub default: Option<toml::Value>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq, Serialize)]
pub enum ContainerType {
    #[serde(rename = "vec")]
    Vec,
    #[serde(rename = "option")]
    Option,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum ComponentType {
    Item(ItemPathBuf),
    Contained {
        #[serde(rename = "type")]
        #[serde(alias = "container_type")]
        #[serde(alias = "container-type")]
        type_: ContainerType,
        #[serde(alias = "element-type")]
        element_type: ItemPathBuf,
    },
}
impl std::fmt::Debug for ComponentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Item(arg0) => write!(f, "{}", arg0),
            Self::Contained {
                type_,
                element_type,
            } => write!(f, "{:?}<{}>", type_, element_type),
        }
    }
}
