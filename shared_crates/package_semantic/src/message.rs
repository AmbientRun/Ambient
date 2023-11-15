use ambient_package::{ItemPathBuf, SnakeCaseIdentifier};
use anyhow::Context as AnyhowContext;
use indexmap::IndexMap;

use crate::{
    Item, ItemData, ItemId, ItemType, ItemVariant, ResolvableItemId, Resolve, Semantic, Type,
};

#[derive(Clone, PartialEq, Debug)]
pub struct Message {
    data: ItemData,

    pub description: Option<String>,
    pub fields: IndexMap<SnakeCaseIdentifier, ResolvableItemId<Type>>,
    pub as_module_message: bool,

    resolved: bool,
}

impl Item for Message {
    const TYPE: ItemType = ItemType::Message;
    type Unresolved = ItemPathBuf;

    fn from_item_value(value: &ItemVariant) -> Option<&Self> {
        match value {
            ItemVariant::Message(value) => Some(value),
            _ => None,
        }
    }

    fn from_item_value_mut(value: &mut ItemVariant) -> Option<&mut Self> {
        match value {
            ItemVariant::Message(value) => Some(value),
            _ => None,
        }
    }

    fn into_item_value(self) -> ItemVariant {
        ItemVariant::Message(self)
    }

    fn data(&self) -> &ItemData {
        &self.data
    }
}
impl Resolve for Message {
    fn resolve(mut self, semantic: &mut Semantic, _self_id: ItemId<Self>) -> anyhow::Result<Self> {
        let parent_id = self.data.parent_id.unwrap();

        let mut fields = IndexMap::new();
        for (name, type_) in &self.fields {
            fields.insert(
                name.clone(),
                match type_ {
                    ResolvableItemId::Unresolved(path) => {
                        let id = semantic.get_contextual_type_id(parent_id, path).with_context(|| {
                            format!("Failed to resolve type `{path:?}` for field `{name}` of message `{}`", self.data.id)
                        })?;
                        ResolvableItemId::Resolved(id)
                    }
                    t => t.clone(),
                },
            );
        }
        self.fields = fields;
        self.resolved = true;

        Ok(self)
    }

    fn already_resolved(&self) -> bool {
        self.resolved
    }
}

impl Message {
    pub(crate) fn from_package(data: ItemData, value: &ambient_package::Message) -> Self {
        Message {
            data,
            description: value.description.clone(),
            fields: value
                .fields
                .iter()
                .map(|(k, v)| (k.clone(), ResolvableItemId::Unresolved(v.clone())))
                .collect(),
            as_module_message: value.as_module_message,
            resolved: false,
        }
    }
}
