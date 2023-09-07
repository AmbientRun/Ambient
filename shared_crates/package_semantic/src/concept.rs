use ambient_package::{Identifier, ItemPathBuf};
use anyhow::Context as AnyhowContext;
use indexmap::IndexMap;

use crate::{
    Component, Item, ItemData, ItemId, ItemType, ItemValue, ResolvableItemId, ResolvableValue,
    Resolve, Scope, Semantic,
};

type ComponentMap = IndexMap<ResolvableItemId<Component>, ConceptValue>;

#[derive(Clone, PartialEq, Debug)]
pub struct Concept {
    data: ItemData,
    resolved: bool,

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
    fn resolve(mut self, semantic: &mut Semantic, _self_id: ItemId<Self>) -> anyhow::Result<Self> {
        let parent_id = self.data.parent_id.unwrap();

        let mut extends = vec![];
        for extend in &self.extends {
            extends.push(match extend {
                ResolvableItemId::Unresolved(path) => semantic
                    .get_contextual_concept_id(parent_id, path.as_path())
                    .map_err(|e| e.into_owned())
                    .with_context(|| {
                        format!(
                            "Failed to resolve concept `{}` for concept `{}`",
                            path, self.data.id
                        )
                    })?,
                ResolvableItemId::Resolved(id) => *id,
            });
        }
        self.extends = extends
            .iter()
            .copied()
            .map(ResolvableItemId::Resolved)
            .collect();

        let component_extractors: [fn(&mut Concept) -> &mut ComponentMap; 2] = [
            |c| &mut c.required_components,
            |c| &mut c.optional_components,
        ];

        let our_id = self.data.id.clone();
        for extractor in component_extractors {
            let mut new_components = ComponentMap::new();

            // Add all components from our extended concepts
            for extend_id in &extends {
                let extend = semantic.resolve(*extend_id)?;
                let extend_components = extractor(extend);
                new_components.extend(
                    extend_components
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone())),
                );
            }

            let components = extractor(&mut self);

            // Add our components
            new_components.extend(components.iter().map(|(k, v)| (k.clone(), v.clone())));

            // Resolve anything that needs to be resolved
            *components = resolve_components(parent_id, &our_id, semantic, &new_components)?;
        }

        self.resolved = true;

        Ok(self)
    }

    fn already_resolved(&self) -> bool {
        self.resolved
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
            resolved: false,
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
    parent_id: ItemId<Scope>,
    concept_id: &Identifier,
    semantic: &mut Semantic,
    unresolved_components: &ComponentMap,
) -> anyhow::Result<ComponentMap> {
    let mut components = IndexMap::new();
    for (resolvable_component, resolvable_value) in unresolved_components {
        let component_id = match resolvable_component {
            ResolvableItemId::Unresolved(path) => semantic
                .get_contextual_component_id(parent_id, path.as_path())
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
            let component = semantic.resolve(component_id)?;
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
            suggested.resolve_in_place(&semantic.items, component_type)?;
        }
        components.insert(ResolvableItemId::Resolved(component_id), value);
    }

    Ok(components)
}
