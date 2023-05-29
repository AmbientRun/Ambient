use ambient_project::{Identifier, ItemPathBuf};
use anyhow::Context as AnyhowContext;
use indexmap::IndexMap;

use crate::{
    Context, Item, ItemData, ItemId, ItemMap, ItemType, ItemValue, ResolvableItemId, Resolve, Type,
};

#[derive(Clone, PartialEq, Debug)]
pub struct Message {
    data: ItemData,

    pub description: Option<String>,
    pub fields: IndexMap<Identifier, ResolvableItemId<Type>>,
}

impl Item for Message {
    const TYPE: ItemType = ItemType::Message;
    type Unresolved = ItemPathBuf;

    fn from_item_value(value: &ItemValue) -> Option<&Self> {
        match value {
            ItemValue::Message(value) => Some(value),
            _ => None,
        }
    }

    fn from_item_value_mut(value: &mut ItemValue) -> Option<&mut Self> {
        match value {
            ItemValue::Message(value) => Some(value),
            _ => None,
        }
    }

    fn into_item_value(self) -> ItemValue {
        ItemValue::Message(self)
    }

    fn data(&self) -> &ItemData {
        &self.data
    }
}
impl Resolve for Message {
    fn resolve(
        &mut self,
        items: &ItemMap,
        _self_id: ItemId<Self>,
        context: &Context,
    ) -> anyhow::Result<()> {
        let mut fields = IndexMap::new();
        for (name, type_) in &self.fields {
            fields.insert(
                name.clone(),
                match type_ {
                    ResolvableItemId::Unresolved(path) => {
                        let id = context.get_type_id(items, path).with_context(|| {
                            format!("Failed to resolve type `{path:?}` for field `{name}` of message `{}`", self.data.id)
                        })?;
                        ResolvableItemId::Resolved(id)
                    }
                    t => t.clone(),
                },
            );
        }
        self.fields = fields;

        Ok(())
    }
}

impl Message {
    pub(crate) fn from_project(data: ItemData, value: &ambient_project::Message) -> Self {
        Message {
            data,
            description: value.description.clone(),
            fields: value
                .fields
                .iter()
                .map(|(k, v)| (k.clone(), ResolvableItemId::Unresolved(v.clone())))
                .collect(),
        }
    }
}
