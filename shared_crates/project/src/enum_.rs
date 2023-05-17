use serde::{Deserialize, Serialize};

use crate::CamelCaseIdentifier;

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Enum(pub Vec<CamelCaseIdentifier>);
