use ambient_package::{
    ComponentType, ItemPath, ItemPathBuf, PascalCaseIdentifier, SnakeCaseIdentifier,
};
use indexmap::IndexMap;
use thiserror::Error;

use crate::{
    Attribute, Component, Concept, Item, ItemData, ItemId, ItemMap, ItemType, ItemValue, Message,
    Package, Resolve, StandardDefinitions, Type,
};

#[derive(Clone, PartialEq)]
pub struct Scope {
    pub data: ItemData,

    pub imports: IndexMap<SnakeCaseIdentifier, ItemId<Package>>,
    pub scopes: IndexMap<SnakeCaseIdentifier, ItemId<Scope>>,
    pub components: IndexMap<SnakeCaseIdentifier, ItemId<Component>>,
    pub concepts: IndexMap<PascalCaseIdentifier, ItemId<Concept>>,
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
impl Resolve for Scope {
    fn resolve(
        self,
        items: &mut ItemMap,
        context: &Context,
        definitions: &StandardDefinitions,
        self_id: ItemId<Self>,
    ) -> anyhow::Result<Self> {
        fn resolve<T: Resolve, U>(
            items: &mut ItemMap,
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
            items.resolve(&context, definitions, *id)?;
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
            imports: Default::default(),
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
                visit_recursive_inner(&items.get(scope), items, visitor)?;
            }

            Ok(())
        }

        visit_recursive_inner(self, items, &mut visitor)
    }
}

#[derive(Error, Debug)]
pub enum ContextGetError<'a> {
    #[error("Failed to find {path} ({type_})")]
    NotFound { path: ItemPath<'a>, type_: ItemType },
}
impl ContextGetError<'_> {
    pub fn into_owned(self) -> ContextGetOwnedError {
        self.into()
    }
}
#[derive(Error, Debug)]
pub enum ContextGetOwnedError {
    #[error("Failed to find {path} ({type_})")]
    NotFound { path: ItemPathBuf, type_: ItemType },
}
impl From<ContextGetError<'_>> for ContextGetOwnedError {
    fn from(error: ContextGetError) -> Self {
        match error {
            ContextGetError::NotFound { path, type_ } => Self::NotFound {
                path: path.to_owned(),
                type_,
            },
        }
    }
}

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
                    if let Some(id) = get_type_id(items, scope_id, id.as_path()) {
                        return Some(id);
                    }
                }
                ComponentType::Contained {
                    type_,
                    element_type,
                } => {
                    if let Some(id) = get_type_id(items, scope_id, element_type.as_path()) {
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

    pub(crate) fn get_attribute_id<'a>(
        &self,
        items: &ItemMap,
        path: ItemPath<'a>,
    ) -> Result<ItemId<Attribute>, ContextGetError<'a>> {
        for &scope_id in self.0.iter().rev() {
            if let Some(id) = get_attribute_id(items, scope_id, path) {
                return Ok(id);
            }
        }
        Err(ContextGetError::NotFound {
            path,
            type_: ItemType::Attribute,
        })
    }

    pub(crate) fn get_concept_id<'a>(
        &self,
        items: &ItemMap,
        path: ItemPath<'a>,
    ) -> Result<ItemId<Concept>, ContextGetError<'a>> {
        for &scope_id in self.0.iter().rev() {
            if let Some(id) = get_concept_id(items, scope_id, path) {
                return Ok(id);
            }
        }
        Err(ContextGetError::NotFound {
            path,
            type_: ItemType::Concept,
        })
    }

    pub(crate) fn get_component_id<'a>(
        &self,
        items: &ItemMap,
        path: ItemPath<'a>,
    ) -> Result<ItemId<Component>, ContextGetError<'a>> {
        for &scope_id in self.0.iter().rev() {
            if let Some(id) = get_component_id(items, scope_id, path) {
                return Ok(id);
            }
        }
        Err(ContextGetError::NotFound {
            path,
            type_: ItemType::Component,
        })
    }
}

fn get_type_id(
    items: &ItemMap,
    self_scope_id: ItemId<Scope>,
    path: ItemPath,
) -> Option<ItemId<Type>> {
    let (scope, item) = path.scope_and_item();
    items
        .get_scope(self_scope_id, scope)
        .ok()?
        .types
        .get(item.as_pascal().ok()?)
        .copied()
}

fn get_attribute_id(
    items: &ItemMap,
    self_scope_id: ItemId<Scope>,
    path: ItemPath,
) -> Option<ItemId<Attribute>> {
    let (scope, item) = path.scope_and_item();
    items
        .get_scope(self_scope_id, scope)
        .ok()?
        .attributes
        .get(item.as_pascal().ok()?)
        .copied()
}

fn get_concept_id(
    items: &ItemMap,
    self_scope_id: ItemId<Scope>,
    path: ItemPath,
) -> Option<ItemId<Concept>> {
    let (scope, item) = path.scope_and_item();
    items
        .get_scope(self_scope_id, scope)
        .ok()?
        .concepts
        .get(item.as_pascal().ok()?)
        .copied()
}

fn get_component_id(
    items: &ItemMap,
    self_scope_id: ItemId<Scope>,
    path: ItemPath,
) -> Option<ItemId<Component>> {
    let (scope, item) = path.scope_and_item();
    items
        .get_scope(self_scope_id, scope)
        .ok()?
        .components
        .get(item.as_snake().ok()?)
        .copied()
}
