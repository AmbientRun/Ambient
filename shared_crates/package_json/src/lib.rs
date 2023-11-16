//! Defines the structure of `ambient_package.json` files.
//!
//! This is a stable format that includes all of the information required to describe a package,
//! its items, and its dependencies. It is intentionally simplified from the internal package
//! semantic representations to allow for easy parsing and manipulation by external tools.

use std::collections::HashMap;

use ambient_primitive_component_definitions::primitive_component_definitions;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

mod item;
use item::impl_item_for_type;
pub use item::{ErasedItemId, Item, ItemData, ItemId, ItemSource, ItemVariant};

mod value;
pub use value::*;

/// Some kind of identifier. Can be snake_case or PascalCase.
pub type Identifier = String;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Manifest {
    pub main_package_id: ItemId<Package>,
    pub root_scope_id: ItemId<Scope>,
    pub items: HashMap<ErasedItemId, ItemVariant>,
}
impl Manifest {
    pub fn get<T: Item>(&self, id: &ItemId<T>) -> &T {
        T::from_item_variant(self.items.get(&id.0).unwrap()).unwrap()
    }

    pub fn packages(&self) -> impl Iterator<Item = (ItemId<Package>, &Package)> {
        self.items.iter().filter_map(|(k, v)| {
            let package = Package::from_item_variant(v)?;
            Some((ItemId::forge(k.clone()), package))
        })
    }

    pub fn main_package(&self) -> &Package {
        self.get(&self.main_package_id)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Package {
    pub data: ItemData,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub repository: Option<String>,
    pub ambient_version: Option<String>,
    pub scope_id: ItemId<Scope>,
    pub dependencies: IndexMap<Identifier, Dependency>,
}
impl_item_for_type!(Package);
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Dependency {
    pub id: ItemId<Package>,
    pub enabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Scope {
    pub data: ItemData,
    pub imports: IndexMap<Identifier, ItemId<Package>>,
    pub scopes: IndexMap<Identifier, ItemId<Scope>>,
    pub components: IndexMap<Identifier, ItemId<Component>>,
    pub concepts: IndexMap<Identifier, ItemId<Concept>>,
    pub messages: IndexMap<Identifier, ItemId<Message>>,
    pub types: IndexMap<Identifier, ItemId<Type>>,
    pub attributes: IndexMap<Identifier, ItemId<Attribute>>,
}
impl_item_for_type!(Scope);

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Component {
    pub data: ItemData,
    pub name: Option<String>,
    pub description: Option<String>,
    pub type_: ItemId<Type>,
    pub attributes: Vec<ItemId<Attribute>>,
    pub default: Option<Value>,
}
impl_item_for_type!(Component);

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Concept {
    pub data: ItemData,
    pub name: Option<String>,
    pub description: Option<String>,
    pub extends: Vec<ItemId<Concept>>,
    pub required_components: IndexMap<ItemId<Component>, ConceptValue>,
    pub optional_components: IndexMap<ItemId<Component>, ConceptValue>,
}
impl_item_for_type!(Concept);
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ConceptValue {
    pub description: Option<String>,
    pub suggested: Option<Value>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Message {
    pub data: ItemData,
    pub description: Option<String>,
    pub fields: IndexMap<Identifier, ItemId<Type>>,
}
impl_item_for_type!(Message);

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Type {
    pub data: ItemData,
    pub inner: TypeInner,
}
impl_item_for_type!(Type);
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(tag = "type", content = "value")]
pub enum TypeInner {
    Primitive(PrimitiveType),
    Vec(ItemId<Type>),
    Option(ItemId<Type>),
    Enum(Enum),
}
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Enum {
    pub description: Option<String>,
    pub members: IndexMap<Identifier, String>,
}
macro_rules! define_primitive_type {
    ($(($value:ident, $type:ty)),*) => {
        #[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
        pub enum PrimitiveType {
            $(
                #[doc = stringify!($type)]
                $value,
            )*
        }

        impl std::fmt::Display for PrimitiveType {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$value => write!(f, stringify!($type)),
                    )*
                }
            }
        }
    }
}
primitive_component_definitions!(define_primitive_type);

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Attribute {
    pub data: ItemData,
}
impl_item_for_type!(Attribute);
