use ambient_project::{Identifier, ItemPathBuf};

use crate::{Context, Item, ItemMap, ItemType, ItemValue};

#[derive(Clone, PartialEq, Debug)]
pub struct Attribute {
    pub id: Identifier,
}
impl Item for Attribute {
    const TYPE: ItemType = ItemType::Attribute;
    type Unresolved = ItemPathBuf;

    fn from_item_value(value: &ItemValue) -> Option<&Self> {
        match value {
            ItemValue::Attribute(value) => Some(value),
            _ => None,
        }
    }

    fn from_item_value_mut(value: &mut ItemValue) -> Option<&mut Self> {
        match value {
            ItemValue::Attribute(value) => Some(value),
            _ => None,
        }
    }

    fn into_item_value(self) -> ItemValue {
        ItemValue::Attribute(self)
    }

    fn resolve(&mut self, _items: &mut ItemMap, _context: &Context) -> anyhow::Result<Self> {
        Ok(self.clone())
    }
}
