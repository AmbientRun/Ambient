use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::Identifier;

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Enum(pub IndexMap<Identifier, String>);
