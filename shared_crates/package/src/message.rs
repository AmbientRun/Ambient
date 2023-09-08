use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{ComponentType, SnakeCaseIdentifier};

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct Message {
    pub description: Option<String>,
    pub fields: IndexMap<SnakeCaseIdentifier, ComponentType>,
    /// When set, will generate a `ModuleMessage` instead of a `RuntimeMessage`.
    ///
    /// Only applicable to messages defined in the `ambient_core` schema. Intentionally undocumented.
    #[serde(default)]
    pub as_module_message: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn does_message_preserve_order_of_fields() {
        let t = r#"
        [fields]
        a = "a"
        c = "c"
        b = "b"
        "#;

        let message: Message = toml::from_str(t).unwrap();

        assert_eq!(
            message.fields.keys().collect::<Vec<_>>(),
            vec![
                &SnakeCaseIdentifier::new("a").unwrap(),
                &SnakeCaseIdentifier::new("c").unwrap(),
                &SnakeCaseIdentifier::new("b").unwrap(),
            ]
        );
    }
}
