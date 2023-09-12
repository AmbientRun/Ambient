use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::ItemPathBuf;

#[derive(Deserialize, Debug, Clone, PartialEq, Serialize)]
pub struct Concept {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub extends: Vec<ItemPathBuf>,
    pub components: Components,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Serialize)]
pub struct Components {
    pub required: IndexMap<ItemPathBuf, ConceptValue>,
    #[serde(default)]
    pub optional: IndexMap<ItemPathBuf, ConceptValue>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Serialize, Default)]
pub struct ConceptValue {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub suggested: Option<toml::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cannot_use_optional_as_component_name() {
        let concept = r#"
        [components.required]
        optional = {}
        "#;

        assert_eq!(
            toml::from_str::<Concept>(concept).unwrap_err().message(),
            "the item identifier `optional` is not a valid snake_case identifier (identifier `optional` is reserved) or a valid PascalCase identifier (identifier `optional` must start with an uppercase ASCII character)"
        );
    }
}
