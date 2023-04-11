use std::collections::BTreeMap;

use serde::Deserialize;

use crate::IdentifierPathBuf;

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Concept {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub extends: Vec<IdentifierPathBuf>,
    pub components: BTreeMap<IdentifierPathBuf, toml::Value>,
}
