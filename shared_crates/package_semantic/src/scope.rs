use ambient_package::{ComponentType, ItemPath, PascalCaseIdentifier, SnakeCaseIdentifier};
use anyhow::Context as AnyhowContext;
use indexmap::IndexMap;

use crate::{
    Attribute, Component, Concept, Item, ItemData, ItemId, ItemMap, ItemType, ItemValue, Message,
    Resolve, ResolveClone, StandardDefinitions, Type,
};

#[derive(Clone, PartialEq, Debug)]
pub struct Context(Vec<ItemId<Scope>>);
impl Context {
    pub(crate) fn new(root_scope: ItemId<Scope>) -> Self {
        Self(vec![root_scope])
    }

    fn push(&mut self, scope: ItemId<Scope>) {
        self.0.push(scope);
    }

    pub(crate) fn get_type_id(
        &self,
        items: &ItemMap,
        component_type: &ComponentType,
    ) -> Option<ItemId<Type>> {
        for &scope_id in self.0.iter().rev() {
            match component_type {
                ComponentType::Item(id) => {
                    if let Ok(id) = get_type_id(items, scope_id, id.as_path()) {
                        return Some(id);
                    }
                }
                ComponentType::Contained {
                    type_,
                    element_type,
                } => {
                    if let Ok(id) = get_type_id(items, scope_id, element_type.as_path()) {
                        return Some(match type_ {
                            ambient_package::ContainerType::Vec => items.get_vec_id(id),
                            ambient_package::ContainerType::Option => items.get_option_id(id),
                        });
                    }
                }
            }
        }
        None
    }

    pub(crate) fn get_attribute_id(
        &self,
        items: &ItemMap,
        path: ItemPath,
    ) -> anyhow::Result<ItemId<Attribute>> {
        for &scope_id in self.0.iter().rev() {
            if let Ok(id) = get_attribute_id(items, scope_id, path) {
                return Ok(id);
            }
        }
        anyhow::bail!("failed to find attribute {:?}", path);
    }

    pub(crate) fn get_concept_id(
        &self,
        items: &ItemMap,
        path: ItemPath,
    ) -> anyhow::Result<ItemId<Concept>> {
        for &scope_id in self.0.iter().rev() {
            if let Ok(id) = get_concept_id(items, scope_id, path) {
                return Ok(id);
            }
        }
        anyhow::bail!("failed to find concept {:?}", path);
    }

    pub(crate) fn get_component_id(
        &self,
        items: &ItemMap,
        path: ItemPath,
    ) -> anyhow::Result<ItemId<Component>> {
        for &scope_id in self.0.iter().rev() {
            if let Ok(id) = get_component_id(items, scope_id, path) {
                return Ok(id);
            }
        }
        anyhow::bail!("failed to find component {:?}", path);
    }
}

#[derive(Clone, PartialEq)]
pub struct Scope {
    pub data: ItemData,

    pub scopes: IndexMap<SnakeCaseIdentifier, ItemId<Scope>>,
    pub components: IndexMap<SnakeCaseIdentifier, ItemId<Component>>,
    pub concepts: IndexMap<SnakeCaseIdentifier, ItemId<Concept>>,
    pub messages: IndexMap<PascalCaseIdentifier, ItemId<Message>>,
    pub types: IndexMap<PascalCaseIdentifier, ItemId<Type>>,
    pub attributes: IndexMap<PascalCaseIdentifier, ItemId<Attribute>>,
}
impl std::fmt::Debug for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ds = f.debug_struct("Scope");
        ds.field("data", &self.data);

        if !self.scopes.is_empty() {
            ds.field("scopes", &self.scopes);
        }
        if !self.components.is_empty() {
            ds.field("components", &self.components);
        }
        if !self.concepts.is_empty() {
            ds.field("concepts", &self.concepts);
        }
        if !self.messages.is_empty() {
            ds.field("messages", &self.messages);
        }
        if !self.types.is_empty() {
            ds.field("types", &self.types);
        }
        if !self.attributes.is_empty() {
            ds.field("attributes", &self.attributes);
        }

        ds.finish()
    }
}
impl Item for Scope {
    const TYPE: ItemType = ItemType::Scope;

    type Unresolved = ();

    fn from_item_value(value: &ItemValue) -> Option<&Self> {
        match value {
            ItemValue::Scope(value) => Some(value),
            _ => None,
        }
    }

    fn from_item_value_mut(value: &mut ItemValue) -> Option<&mut Self> {
        match value {
            ItemValue::Scope(value) => Some(value),
            _ => None,
        }
    }

    fn into_item_value(self) -> ItemValue {
        ItemValue::Scope(self)
    }

    fn data(&self) -> &ItemData {
        &self.data
    }
}
/// Scope uses `ResolveClone` because scopes can be accessed during resolution
/// of their children, so we need to clone the scope to avoid a double-borrow.
impl ResolveClone for Scope {
    fn resolve_clone(
        self,
        items: &mut ItemMap,
        context: &Context,
        definitions: &StandardDefinitions,
        self_id: ItemId<Self>,
    ) -> anyhow::Result<Self> {
        fn resolve<T: Resolve, U>(
            items: &ItemMap,
            context: &Context,
            definitions: &StandardDefinitions,
            item_ids: &IndexMap<U, ItemId<T>>,
        ) -> anyhow::Result<()> {
            for id in item_ids.values().copied() {
                items.resolve(context, definitions, id)?;
            }

            Ok(())
        }

        let mut context = context.clone();
        context.push(self_id);

        for id in self.scopes.values() {
            items.resolve_clone(&context, definitions, *id)?;
        }
        resolve(items, &context, definitions, &self.components)?;
        resolve(items, &context, definitions, &self.concepts)?;
        resolve(items, &context, definitions, &self.messages)?;
        resolve(items, &context, definitions, &self.types)?;
        resolve(items, &context, definitions, &self.attributes)?;

        Ok(self)
    }
}
impl Scope {
    /// Creates a new empty scope with the specified data.
    pub fn new(data: ItemData) -> Self {
        Self {
            data,
            scopes: Default::default(),
            components: Default::default(),
            concepts: Default::default(),
            messages: Default::default(),
            types: Default::default(),
            attributes: Default::default(),
        }
    }

    pub fn visit_recursive(
        &self,
        items: &ItemMap,
        mut visitor: impl FnMut(&Scope) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        fn visit_recursive_inner(
            scope: &Scope,
            items: &ItemMap,
            visitor: &mut dyn FnMut(&Scope) -> anyhow::Result<()>,
        ) -> anyhow::Result<()> {
            visitor(scope)?;

            for scope in scope.scopes.values().copied() {
                visit_recursive_inner(&*items.get(scope)?, items, visitor)?;
            }

            Ok(())
        }

        visit_recursive_inner(self, items, &mut visitor)
    }
}

fn get_type_id(
    items: &ItemMap,
    self_scope_id: ItemId<Scope>,
    path: ItemPath,
) -> anyhow::Result<ItemId<Type>> {
    let (scope, item) = path.scope_and_item();
    let item = item.as_pascal().context("type name must be PascalCase")?;
    items
        .get_scope(self_scope_id, scope.iter())?
        .types
        .get(item)
        .copied()
        .with_context(|| format!("failed to find type {item} in {scope:?}"))
}

fn get_attribute_id(
    items: &ItemMap,
    self_scope_id: ItemId<Scope>,
    path: ItemPath,
) -> anyhow::Result<ItemId<Attribute>> {
    let (scope, item) = path.scope_and_item();
    let item = item
        .as_pascal()
        .context("attribute name must be PascalCase")?;
    items
        .get_scope(self_scope_id, scope.iter())?
        .attributes
        .get(item)
        .copied()
        .with_context(|| format!("failed to find attribute {item} in {scope:?}",))
}

fn get_concept_id(
    items: &ItemMap,
    self_scope_id: ItemId<Scope>,
    path: ItemPath,
) -> anyhow::Result<ItemId<Concept>> {
    let (scope, item) = path.scope_and_item();
    let item = item.as_snake().context("concept name must be snake_case")?;
    items
        .get_scope(self_scope_id, scope.iter())?
        .concepts
        .get(item)
        .copied()
        .with_context(|| format!("failed to find concept {item} in {scope:?}",))
}

fn get_component_id(
    items: &ItemMap,
    self_scope_id: ItemId<Scope>,
    path: ItemPath,
) -> anyhow::Result<ItemId<Component>> {
    let (scope, item) = path.scope_and_item();
    let item = item.as_snake().context("concept name must be snake_case")?;
    items
        .get_scope(self_scope_id, scope.iter())?
        .components
        .get(item)
        .copied()
        .with_context(|| format!("failed to find component {item} in {scope:?}",))
}
