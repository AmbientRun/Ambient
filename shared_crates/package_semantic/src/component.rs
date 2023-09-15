use ambient_package::ItemPathBuf;
use anyhow::Context as AnyhowContext;

use crate::{
    Attribute, Item, ItemData, ItemId, ItemType, ItemValue, ResolvableItemId, ResolvableValue,
    Resolve, Semantic, Type,
};

#[derive(Clone, PartialEq, Debug)]
pub struct Component {
    pub data: ItemData,

    pub name: Option<String>,
    pub description: Option<String>,
    pub type_: ResolvableItemId<Type>,
    pub attributes: Vec<ResolvableItemId<Attribute>>,
    pub default: Option<ResolvableValue>,

    resolved: bool,
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

    fn data(&self) -> &ItemData {
        &self.data
    }
}
impl Resolve for Component {
    fn resolve(mut self, semantic: &mut Semantic, _self_id: ItemId<Self>) -> anyhow::Result<Self> {
        let parent_id = self.data.parent_id.unwrap();

        let type_id = match &self.type_ {
            ResolvableItemId::Unresolved(ty) => semantic
                .get_contextual_type_id(parent_id, ty)
                .with_context(|| {
                    format!(
                        "Failed to resolve type `{ty:?}` for component `{}`",
                        self.data.id
                    )
                })?,
            ResolvableItemId::Resolved(id) => *id,
        };
        self.type_ = ResolvableItemId::Resolved(type_id);

        let mut attributes = vec![];
        for attribute in &self.attributes {
            attributes.push(match attribute {
                ResolvableItemId::Unresolved(path) => {
                    let id = semantic
                        .get_contextual_attribute_id(parent_id, path.as_path())
                        .map_err(|e| e.into_owned())
                        .with_context(|| {
                            format!(
                                "Failed to resolve attribute `{path}` for component `{}`",
                                self.data.id
                            )
                        })?;
                    ResolvableItemId::Resolved(id)
                }
                t => t.clone(),
            });
        }

        // If this is an enum, emit the `Enum` attribute
        if semantic.items.get(type_id).inner.as_enum().is_some() {
            attributes.push(ResolvableItemId::Resolved(
                semantic.standard_definitions.attributes.enum_,
            ));
        }
        self.attributes = attributes;

        if let Some(default) = &mut self.default {
            default.resolve_in_place(&semantic.items, type_id)?;
        }

        self.resolved = true;

        Ok(self)
    }

    fn already_resolved(&self) -> bool {
        self.resolved
    }
}
impl Component {
    pub(crate) fn from_package(data: ItemData, value: &ambient_package::Component) -> Self {
        Self {
            data,
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
            resolved: false,
        }
    }
}
