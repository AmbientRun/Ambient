use ambient_project::ItemPathBuf;

use crate::{
    Context, Item, ItemData, ItemId, ItemMap, ItemType, ItemValue, Resolve, StandardDefinitions,
};

#[derive(Clone, PartialEq, Debug)]
pub struct Attribute {
    pub data: ItemData,
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

    fn data(&self) -> &ItemData {
        &self.data
    }
}
impl Resolve for Attribute {
    fn resolve(
        &mut self,
        _items: &ItemMap,
        _context: &Context,
        _definitions: &StandardDefinitions,
        _self_id: ItemId<Self>,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
