use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::IdentifierPathBuf;

#[derive(Deserialize, Debug, Clone, PartialEq, Serialize)]
pub struct Concept {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub extends: Vec<IdentifierPathBuf>,
    pub components: IndexMap<IdentifierPathBuf, toml::Value>,
}
