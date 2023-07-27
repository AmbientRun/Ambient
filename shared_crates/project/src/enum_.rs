use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::PascalCaseIdentifier;

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Enum(pub IndexMap<PascalCaseIdentifier, String>);
