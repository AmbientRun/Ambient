use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::ItemPathBuf;

#[derive(Deserialize, Debug, Clone, PartialEq, Serialize)]
pub struct Concept {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub extends: Vec<ItemPathBuf>,
    pub components: IndexMap<ItemPathBuf, toml::Value>,
}
