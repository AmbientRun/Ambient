use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::PascalCaseIdentifier;

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
pub struct Enum {
    #[serde(default)]
    pub description: Option<String>,
    pub members: IndexMap<PascalCaseIdentifier, String>,
}
