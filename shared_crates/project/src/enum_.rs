use serde::{Deserialize, Serialize};

use crate::Identifier;

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Enum(pub Vec<EnumMember>);

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
pub struct EnumMember {
    pub name: Identifier,
    pub description: Option<String>,
}
