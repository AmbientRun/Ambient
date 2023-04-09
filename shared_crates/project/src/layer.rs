use serde::Deserialize;

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Layer {
    pub name: String,
    pub description: String,
}