use ambient_package::ItemPathBuf;
use anyhow::Context as AnyhowContext;
use indexmap::IndexMap;

use crate::{
    Component, Context, Item, ItemData, ItemId, ItemMap, ItemType, ItemValue, ResolvableItemId,
    ResolvableValue, Resolve, StandardDefinitions,
};

#[derive(Clone, PartialEq, Debug)]
pub struct Concept {
    data: ItemData,

    pub name: Option<String>,
    pub description: Option<String>,
    pub extends: Vec<ResolvableItemId<Concept>>,
    pub components: IndexMap<ResolvableItemId<Component>, ResolvableValue>,
}
impl Item for Concept {
    const TYPE: ItemType = ItemType::Concept;
    type Unresolved = ItemPathBuf;

    fn from_item_value(value: &ItemValue) -> Option<&Self> {
        match value {
            ItemValue::Concept(value) => Some(value),
            _ => None,
        }
    }

    fn from_item_value_mut(value: &mut ItemValue) -> Option<&mut Self> {
        match value {
            ItemValue::Concept(value) => Some(value),
            _ => None,
        }
    }

    fn into_item_value(self) -> ItemValue {
        ItemValue::Concept(self)
    }

    fn data(&self) -> &ItemData {
        &self.data
    }
}
impl Resolve for Concept {
    fn resolve(
        mut self,
        items: &mut ItemMap,
        context: &Context,
        definitions: &StandardDefinitions,
        _self_id: ItemId<Self>,
    ) -> anyhow::Result<Self> {
        let mut extends = vec![];
        for extend in &self.extends {
            extends.push(match extend {
                ResolvableItemId::Unresolved(path) => {
                    let id = context
                        .get_concept_id(items, path.as_path())
                        .map_err(|e| e.into_owned())
                        .with_context(|| {
                            format!(
                                "Failed to resolve concept `{}` for concept `{}",
                                path, self.data.id
                            )
                        })?;
                    ResolvableItemId::Resolved(id)
                }
                t => t.clone(),
            });
        }
        self.extends = extends;

        let mut components = IndexMap::new();
        for (resolvable_component, resolvable_value) in &self.components {
            let component_id = match resolvable_component {
                ResolvableItemId::Unresolved(path) => context
                    .get_component_id(items, path.as_path())
                    .map_err(|e| e.into_owned())
                    .with_context(|| {
                        format!(
                            "Failed to get component `{}` for concept `{}",
                            path, self.data.id
                        )
                    })?,
                ResolvableItemId::Resolved(id) => *id,
            };
            let component_type = {
                let component = items.resolve(context, definitions, component_id)?;
                component.type_.as_resolved().with_context(|| {
                    format!(
                        "Failed to get type for component `{}` for concept `{}`",
                        component.data().id,
                        self.data.id
                    )
                })?
            };

            let mut value = resolvable_value.clone();
            value.resolve(items, component_type)?;
            components.insert(ResolvableItemId::Resolved(component_id), value);
        }
        self.components = components;

        Ok(self)
    }
}
impl Concept {
    pub(crate) fn from_package(data: ItemData, value: &ambient_package::Concept) -> Self {
        Concept {
            data,
            name: value.name.clone(),
            description: value.description.clone(),
            extends: value
                .extends
                .iter()
                .map(|v| ResolvableItemId::Unresolved(v.clone()))
                .collect(),
            components: value
                .components
                .iter()
                .map(|(k, v)| {
                    (
                        ResolvableItemId::Unresolved(k.clone()),
                        ResolvableValue::Unresolved(v.clone()),
                    )
                })
                .collect(),
        }
    }
}
