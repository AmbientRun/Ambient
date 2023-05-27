use ambient_project::{Identifier, ItemPathBuf};
use anyhow::Context as AnyhowContext;

use crate::{
    Attribute, Context, Item, ItemId, ItemMap, ItemType, ItemValue, ResolvableItemId,
    ResolvableValue, Resolve, Scope, Type,
};

#[derive(Clone, PartialEq, Debug)]
pub struct Component {
    pub parent: ItemId<Scope>,
    pub id: Identifier,
    pub name: Option<String>,
    pub description: Option<String>,
    pub type_: ResolvableItemId<Type>,
    pub attributes: Vec<ResolvableItemId<Attribute>>,
    pub default: Option<ResolvableValue>,
}
impl Item for Component {
    const TYPE: ItemType = ItemType::Component;
    type Unresolved = ItemPathBuf;

    fn from_item_value(value: &ItemValue) -> Option<&Self> {
        match value {
            ItemValue::Component(value) => Some(value),
            _ => None,
        }
    }

    fn from_item_value_mut(value: &mut ItemValue) -> Option<&mut Self> {
        match value {
            ItemValue::Component(value) => Some(value),
            _ => None,
        }
    }

    fn into_item_value(self) -> ItemValue {
        ItemValue::Component(self)
    }

    fn parent(&self) -> Option<ItemId<Scope>> {
        Some(self.parent)
    }

    fn id(&self) -> &Identifier {
        &self.id
    }
}
impl Resolve for Component {
    fn resolve(
        &mut self,
        items: &ItemMap,
        _self_id: ItemId<Self>,
        context: &Context,
    ) -> anyhow::Result<()> {
        let type_id = match &self.type_ {
            ResolvableItemId::Unresolved(ty) => {
                context.get_type_id(items, &ty).with_context(|| {
                    format!(
                        "Failed to resolve type `{ty:?}` for component `{}`",
                        self.id
                    )
                })?
            }
            ResolvableItemId::Resolved(id) => *id,
        };
        self.type_ = ResolvableItemId::Resolved(type_id);

        let mut attributes = vec![];
        for attribute in &self.attributes {
            attributes.push(match attribute {
                ResolvableItemId::Unresolved(path) => {
                    let id = context
                        .get_attribute_id(items, path.as_path())
                        .with_context(|| {
                            format!(
                                "Failed to resolve attribute `{path}` for component `{}`",
                                self.id
                            )
                        })?;
                    ResolvableItemId::Resolved(id)
                }
                t => t.clone(),
            });
        }
        self.attributes = attributes;

        if let Some(default) = &mut self.default {
            default.resolve(items, type_id)?;
        }

        Ok(())
    }
}
impl Component {
    pub(crate) fn from_project(
        parent: ItemId<Scope>,
        id: Identifier,
        value: &ambient_project::Component,
    ) -> Self {
        Self {
            parent,
            id,
            name: value.name.clone(),
            description: value.description.clone(),
            type_: ResolvableItemId::Unresolved(value.type_.clone()),
            attributes: value
                .attributes
                .iter()
                .map(|attribute| ResolvableItemId::Unresolved(attribute.clone()))
                .collect(),
            default: value
                .default
                .as_ref()
                .map(|v| ResolvableValue::Unresolved(v.clone())),
        }
    }
}
