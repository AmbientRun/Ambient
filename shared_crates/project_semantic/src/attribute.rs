use ambient_project::{Identifier, ItemPathBuf};

use crate::{Context, Item, ItemId, ItemMap, ItemType, ItemValue, Resolve};

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
}
impl Resolve for Attribute {
    fn resolve(
        &mut self,
        _items: &ItemMap,
        _self_id: ItemId<Self>,
        _context: &Context,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
