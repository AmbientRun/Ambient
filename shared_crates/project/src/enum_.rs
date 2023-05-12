use serde::{Deserialize, Serialize};

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Enum(pub Vec<String>);
