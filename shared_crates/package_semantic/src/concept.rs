use ambient_package::{Identifier, ItemPathBuf};
use anyhow::Context as AnyhowContext;
use indexmap::IndexMap;

use crate::{
    Component, Context, Item, ItemData, ItemId, ItemMap, ItemType, ItemValue, ResolvableItemId,
    ResolvableValue, Resolve, StandardDefinitions,
};

type ComponentMap = IndexMap<ResolvableItemId<Component>, ConceptValue>;

#[derive(Clone, PartialEq, Debug)]
pub struct Concept {
    data: ItemData,

    pub name: Option<String>,
    pub description: Option<String>,
    pub extends: Vec<ResolvableItemId<Concept>>,
    pub required_components: ComponentMap,
    pub optional_components: ComponentMap,
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

        for components in [&mut self.required_components, &mut self.optional_components] {
            *components =
                resolve_components(&self.data.id, items, context, definitions, components)?;
        }

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
            required_components: value
                .components
                .required
                .iter()
                .map(|(k, v)| {
                    (
                        ResolvableItemId::Unresolved(k.clone()),
                        ConceptValue::from_package(v),
                    )
                })
                .collect(),
            optional_components: value
                .components
                .optional
                .iter()
                .map(|(k, v)| {
                    (
                        ResolvableItemId::Unresolved(k.clone()),
                        ConceptValue::from_package(v),
                    )
                })
                .collect(),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct ConceptValue {
    pub description: Option<String>,
    pub suggested: Option<ResolvableValue>,
}
impl ConceptValue {
    pub(crate) fn from_package(value: &ambient_package::ConceptValue) -> Self {
        ConceptValue {
            description: value.description.clone(),
            suggested: value.suggested.clone().map(ResolvableValue::Unresolved),
        }
    }
}

fn resolve_components(
    concept_id: &Identifier,
    items: &mut ItemMap,
    context: &Context,
    definitions: &StandardDefinitions,
    unresolved_components: &ComponentMap,
) -> anyhow::Result<ComponentMap> {
    let mut components = IndexMap::new();
    for (resolvable_component, resolvable_value) in unresolved_components {
        let component_id = match resolvable_component {
            ResolvableItemId::Unresolved(path) => context
                .get_component_id(items, path.as_path())
                .map_err(|e| e.into_owned())
                .with_context(|| {
                    format!(
                        "Failed to get component `{}` for concept `{}",
                        path, concept_id
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
                    concept_id
                )
            })?
        };

        let mut value = resolvable_value.clone();
        if let Some(suggested) = value.suggested.as_mut() {
            suggested.resolve_in_place(items, component_type)?;
        }
        components.insert(ResolvableItemId::Resolved(component_id), value);
    }

    Ok(components)
}
