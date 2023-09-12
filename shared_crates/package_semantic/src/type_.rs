use ambient_package::{ComponentType, PascalCaseIdentifier};
use indexmap::IndexMap;

use crate::{Item, ItemData, ItemId, ItemType, ItemValue, PrimitiveType, Resolve, Semantic};

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
impl TypeInner {
    pub fn as_primitive(&self) -> Option<PrimitiveType> {
        match self {
            Self::Primitive(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_enum(&self) -> Option<&Enum> {
        match self {
            Self::Enum(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_vec(&self) -> Option<ItemId<Type>> {
        match self {
            Self::Vec(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_option(&self) -> Option<ItemId<Type>> {
        match self {
            Self::Option(v) => Some(*v),
            _ => None,
        }
    }
}
impl Type {
    pub fn new(data: ItemData, inner: TypeInner) -> Self {
        Self { data, inner }
    }

    pub(crate) fn from_package_enum(data: ItemData, value: &ambient_package::Enum) -> Self {
        Self::new(
            data,
            TypeInner::Enum(Enum {
                description: value.description.clone(),
                members: value.members.clone(),
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
    fn resolve(self, _semantic: &mut Semantic, _self_id: ItemId<Self>) -> anyhow::Result<Self> {
        Ok(self)
    }

    fn already_resolved(&self) -> bool {
        true
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Enum {
    pub description: Option<String>,
    pub members: IndexMap<PascalCaseIdentifier, String>,
}
