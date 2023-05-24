use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{ComponentType, Identifier};

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct Message {
    pub description: Option<String>,
    pub fields: IndexMap<Identifier, ComponentType>,
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

        let message: Message = toml::from_str(&t).unwrap();

        assert_eq!(
            message.fields.keys().collect::<Vec<_>>(),
            vec![
                &Identifier::new("a").unwrap(),
                &Identifier::new("c").unwrap(),
                &Identifier::new("b").unwrap(),
            ]
        );
    }
}
