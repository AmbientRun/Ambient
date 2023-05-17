use serde::{Deserialize, Serialize};

use crate::CamelCaseIdentifier;

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Enum(pub Vec<EnumMember>);

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
pub struct EnumMember {
    pub name: CamelCaseIdentifier,
    pub description: Option<String>,
}
