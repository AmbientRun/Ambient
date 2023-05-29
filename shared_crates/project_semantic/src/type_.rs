use ambient_project::{ComponentType, Identifier};
use indexmap::IndexMap;

use crate::{
    Context, Item, ItemData, ItemId, ItemMap, ItemType, ItemValue, PrimitiveType, Resolve,
};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Type {
    pub data: ItemData,
    pub inner: TypeInner,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TypeInner {
    Primitive(PrimitiveType),
    Vec(ItemId<Type>),
    Option(ItemId<Type>),
    Enum(Enum),
}
impl Type {
    pub fn new(data: ItemData, inner: TypeInner) -> Self {
        Self { data, inner }
    }

    pub(crate) fn from_project_enum(data: ItemData, value: &ambient_project::Enum) -> Self {
        Self::new(
            data,
            TypeInner::Enum(Enum {
                members: value.0.clone(),
            }),
        )
    }
}
impl Item for Type {
    const TYPE: ItemType = ItemType::Type;
    type Unresolved = ComponentType;

    fn from_item_value(value: &ItemValue) -> Option<&Self> {
        match value {
            ItemValue::Type(value) => Some(value),
            _ => None,
        }
    }

    fn from_item_value_mut(value: &mut ItemValue) -> Option<&mut Self> {
        match value {
            ItemValue::Type(value) => Some(value),
            _ => None,
        }
    }

    fn into_item_value(self) -> ItemValue {
        ItemValue::Type(self)
    }

    fn data(&self) -> &ItemData {
        &self.data
    }
}
impl Resolve for Type {
    fn resolve(
        &mut self,
        _items: &ItemMap,
        _self_id: ItemId<Self>,
        _context: &Context,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Enum {
    pub members: IndexMap<Identifier, String>,
}
