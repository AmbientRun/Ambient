use ambient_project::{ComponentType, Identifier};
use indexmap::IndexMap;

use crate::{Context, Item, ItemId, ItemMap, ItemType, ItemValue, PrimitiveType, Resolve, Scope};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Type {
    pub parent: ItemId<Scope>,
    pub id: Identifier,
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
    pub fn new(parent: ItemId<Scope>, id: Identifier, inner: TypeInner) -> Self {
        Self { parent, id, inner }
    }

    pub(crate) fn from_project_enum(
        parent: ItemId<Scope>,
        id: Identifier,
        value: &ambient_project::Enum,
    ) -> Self {
        Self::new(
            parent,
            id,
            TypeInner::Enum(Enum {
                members: value.0.clone(),
            }),
        )
    }

    pub fn to_string(&self, items: &ItemMap) -> anyhow::Result<String> {
        Ok(match &self.inner {
            TypeInner::Primitive(pt) => pt.to_string(),
            TypeInner::Vec(id) => {
                let contained = items.get(*id)?;
                format!("vec<{}>", contained.to_string(items)?)
            }
            TypeInner::Option(id) => {
                let contained = items.get(*id)?;
                format!("option<{}>", contained.to_string(items)?)
            }
            TypeInner::Enum(_) => self.id.to_string(),
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

    fn id(&self) -> &Identifier {
        &self.id
    }

    fn parent(&self) -> Option<ItemId<Scope>> {
        Some(self.parent)
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
