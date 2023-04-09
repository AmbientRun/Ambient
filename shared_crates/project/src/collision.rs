use serde::Deserialize;

use crate::IdentifierPathBuf;

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct CollisionResponse {
    pub layer_a: IdentifierPathBuf,
    pub layer_b: IdentifierPathBuf,
    pub filter: CollisionFilter,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub enum CollisionFilter {
    Block,
    Overlap,
    Ignore,
}
