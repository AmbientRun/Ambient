use std::fmt::Display;

use ambient_project::{ComponentType, Identifier};
use indexmap::IndexMap;

use crate::{
    item::Resolve, Context, Item, ItemId, ItemMap, ItemType, ItemValue, PrimitiveType, Semantic,
};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Type {
    Primitive(PrimitiveType),
    Vec(ItemId<Type>),
    Option(ItemId<Type>),
    Enum(Enum),
}
impl Type {
    pub(crate) fn from_project_enum(id: Identifier, value: &ambient_project::Enum) -> Self {
        Self::Enum(Enum {
            id,
            members: value.0.clone(),
        })
    }

    pub fn to_string(&self, semantic: &Semantic) -> anyhow::Result<String> {
        Ok(match self {
            Type::Primitive(pt) => pt.to_string(),
            Type::Vec(id) => {
                let inner = semantic.items.get_without_resolve(*id)?;
                format!("Vec<{}>", inner.to_string(semantic)?)
            }
            Type::Option(id) => {
                let inner = semantic.items.get_without_resolve(*id)?;
                format!("Option<{}>", inner.to_string(semantic)?)
            }
            Type::Enum(e) => e.to_string(),
        })
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
    pub id: Identifier,
    pub members: IndexMap<Identifier, String>,
}
impl Display for Enum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "enum({})", self.id)
    }
}
