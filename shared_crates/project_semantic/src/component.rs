use ambient_project::{Identifier, ItemPathBuf};

use crate::{
    Attribute, Context, Item, ItemMap, ItemType, ItemValue, ResolvableItemId, ResolvableValue, Type,
};

#[derive(Clone, PartialEq, Debug)]
pub struct Component {
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

    fn resolve(&mut self, items: &mut ItemMap, context: &Context) -> Self {
        let mut new = self.clone();

        new.type_ = match new.type_ {
            ResolvableItemId::Unresolved(path) => {
                let id = context.get_type_id(items, &path).unwrap();
                ResolvableItemId::Resolved(id)
            }
            t => t,
        };

        let mut attributes = vec![];
        for attribute in &new.attributes {
            attributes.push(match attribute {
                ResolvableItemId::Unresolved(path) => {
                    let id = context.get_attribute_id(path.as_path()).unwrap();
                    ResolvableItemId::Resolved(id)
                }
                t => t.clone(),
            });
        }
        new.attributes = attributes;

        if let Some(default) = &mut new.default {
            default.resolve(items, new.type_.as_resolved().unwrap());
        }

        new
    }
}
impl Component {
    pub(crate) fn from_project(id: Identifier, value: &ambient_project::Component) -> Self {
        Self {
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
