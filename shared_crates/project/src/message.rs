use std::collections::BTreeMap;

use serde::Deserialize;

use crate::{ComponentType, Identifier};

#[derive(Clone, PartialEq, Eq, Debug, Deserialize)]
pub struct Message {
    pub name: String,
    pub description: String,
    pub values: BTreeMap<Identifier, ComponentType>,
}
