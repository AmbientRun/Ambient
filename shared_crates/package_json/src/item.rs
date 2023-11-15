use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

use crate::{Attribute, Component, Concept, Message, Package, Scope, Type};

/// A system identifier for an item without any type-specific information. Consider using [ItemId] instead.
pub type ErasedItemId = String;

pub trait Item {
    fn data(&self) -> &ItemData;
    fn from_item_value(value: &ItemVariant) -> Option<&Self>;
    fn into_item_value(self) -> ItemVariant;
}

macro_rules! impl_item_for_type {
    ($ty:ident) => {
        impl Item for $ty {
            fn data(&self) -> &ItemData {
                &self.data
            }

            fn from_item_value(value: &ItemVariant) -> Option<&Self> {
                match value {
                    ItemVariant::$ty(item) => Some(item),
                    _ => None,
                }
            }

            fn into_item_value(self) -> ItemVariant {
                ItemVariant::$ty(self)
            }
        }
    };
}
pub(crate) use impl_item_for_type;

#[derive(Clone, PartialEq, Debug, Eq, Serialize, Deserialize)]
pub struct ItemData {
    /// The identifier of this item
    pub id: String,
    /// Where this item came from.
    pub source: ItemSource,
}

#[derive(Copy, Clone, PartialEq, Debug, Eq, Serialize, Deserialize)]
pub enum ItemSource {
    /// This is an item defined by the semantic system that should be present in all languages.
    /// Example: `String`, etc.
    System,
    /// This is an item defined by the Ambient API.
    /// Example: `Layout`, etc.
    Ambient,
    /// This is an item defined by the user.
    User,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ItemVariant {
    Package(Package),
    Scope(Scope),
    Component(Component),
    Concept(Concept),
    Message(Message),
    Type(Type),
    Attribute(Attribute),
}

pub struct ItemId<T: Item>(String, PhantomData<T>);
impl<T: Item> ItemId<T> {
    pub fn from_u128(id: u128) -> Self {
        Self(id.to_string(), PhantomData)
    }
}
impl<T: Item> std::hash::Hash for ItemId<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
impl<T: Item> Clone for ItemId<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}
impl<T: Item> Debug for ItemId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ItemId<{}>({:?})", std::any::type_name::<T>(), self.0)
    }
}
impl<T: Item> Display for ItemId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}
impl<T: Item> PartialEq for ItemId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<T: Item> Eq for ItemId<T> {}
impl<T: Item> Serialize for ItemId<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}
impl<'de, T: Item> Deserialize<'de> for ItemId<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer).map(|s| Self(s, PhantomData))
    }
}
