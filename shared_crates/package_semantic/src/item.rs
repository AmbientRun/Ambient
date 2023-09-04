use std::{
    collections::HashMap,
    fmt::{self, Debug, Display},
    marker::PhantomData,
};

use ambient_package::{Identifier, PascalCaseIdentifier, SnakeCaseIdentifier};
use ambient_std::topological_sort::{topological_sort, TopologicalSortable};
use thiserror::Error;
use ulid::Ulid;

use crate::{
    Attribute, Component, Concept, Context, Message, Package, Scope, StandardDefinitions, Type,
    TypeInner,
};

#[derive(Error, Debug)]
pub enum GetScopeError {
    #[error(
        "failed to find scope `{segment}` in scope `{scope_path}` while searching for `{path:?}`"
    )]
    NotFound {
        segment: SnakeCaseIdentifier,
        scope_path: String,
        path: Vec<SnakeCaseIdentifier>,
    },
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct ItemMap {
    items: HashMap<Ulid, ItemValue>,
    vec_items: HashMap<ItemId<Type>, ItemId<Type>>,
    option_items: HashMap<ItemId<Type>, ItemId<Type>>,
}
impl ItemMap {
    pub fn add<T: Item>(&mut self, item: T) -> ItemId<T> {
        if T::TYPE == Type::TYPE {
            let data = item.data().clone();
            let raw_new_id = self.add_raw(item);
            // We know this is a type, so reify it
            let new_id = ItemId(raw_new_id.0, PhantomData::<Type>);

            let vec_id = self.add_raw(Type::new(
                ItemData {
                    id: PascalCaseIdentifier::new(&format!("Vec{}", data.id))
                        .unwrap()
                        .into(),
                    ..data
                },
                TypeInner::Vec(new_id),
            ));
            self.vec_items.insert(new_id, vec_id);

            let option_id = self.add_raw(Type::new(
                ItemData {
                    id: PascalCaseIdentifier::new(&format!("Option{}", data.id))
                        .unwrap()
                        .into(),
                    ..data
                },
                TypeInner::Option(new_id),
            ));
            self.option_items.insert(new_id, option_id);
            raw_new_id
        } else {
            self.add_raw(item)
        }
    }

    // We use ULID creation as this code is only run on the server, where randomness is available.
    // If this runs on the client, then yes, it should panic!
    #[allow(clippy::disallowed_methods)]
    fn add_raw<T: Item>(&mut self, value: T) -> ItemId<T> {
        let ulid = ulid::Ulid::new();
        self.items.insert(ulid, value.into_item_value());
        ItemId(ulid, PhantomData)
    }

    /// Returns a reference to the item with the given id.
    ///
    /// Does not resolve the item.
    pub fn get<T: Item>(&self, id: ItemId<T>) -> &T {
        T::from_item_value(self.items.get(&id.0).unwrap()).unwrap()
    }

    /// Returns a mutable reference to the item with the given id.
    ///
    /// Does not resolve the item.
    pub fn get_mut<T: Item>(&mut self, id: ItemId<T>) -> &mut T {
        T::from_item_value_mut(self.items.get_mut(&id.0).unwrap()).unwrap()
    }

    pub fn insert<T: Item>(&mut self, id: ItemId<T>, item: T) {
        self.items.insert(id.0, item.into_item_value());
    }

    /// Resolve the item with the given id by cloning it, avoiding borrowing issues.
    pub(crate) fn resolve_clone<T: ResolveClone>(
        &mut self,
        context: &Context,
        definitions: &StandardDefinitions,
        id: ItemId<T>,
    ) -> anyhow::Result<&mut T> {
        let item = self.get(id).clone();
        let new_item = item.resolve_clone(self, context, definitions, id)?;
        self.insert(id, new_item);
        Ok(self.get_mut(id))
    }

    pub fn get_vec_id(&self, id: ItemId<Type>) -> ItemId<Type> {
        self.vec_items.get(&id).copied().unwrap()
    }

    pub fn get_option_id(&self, id: ItemId<Type>) -> ItemId<Type> {
        self.option_items.get(&id).copied().unwrap()
    }

    pub fn get_scope_id(
        &self,
        start_scope_id: ItemId<Scope>,
        mut path: &[SnakeCaseIdentifier],
    ) -> Result<ItemId<Scope>, GetScopeError> {
        let mut scope_id = start_scope_id;

        // If the first segment corresponds to an import, use that instead
        if let Some(first_segment) = path.first() {
            if let Some(package_id) = self.get(scope_id).imports.get(first_segment) {
                scope_id = self.get(*package_id).scope_id;
                path = &path[1..];
            }
        }

        for segment in path {
            let scope = self.get(scope_id);
            scope_id =
                scope
                    .scopes
                    .get(segment)
                    .copied()
                    .ok_or_else(|| GetScopeError::NotFound {
                        segment: segment.clone(),
                        scope_path: self.fully_qualified_display_path(&*scope, None, None),
                        path: path.to_vec(),
                    })?;
        }
        Ok(scope_id)
    }

    pub fn get_scope(
        &self,
        start_scope_id: ItemId<Scope>,
        path: &[SnakeCaseIdentifier],
    ) -> anyhow::Result<&Scope> {
        Ok(self.get(self.get_scope_id(start_scope_id, path)?))
    }

    pub(crate) fn get_or_create_scope_mut(
        &mut self,
        start_scope_id: ItemId<Scope>,
        path: &[SnakeCaseIdentifier],
    ) -> &mut Scope {
        let mut scope_id = start_scope_id;
        for segment in path.iter() {
            let existing_id = self.get(scope_id).scopes.get(segment).copied();
            scope_id = match existing_id {
                Some(id) => id,
                None => {
                    let parent_scope_data = self.get(scope_id).data().clone();
                    let new_id = self.add(Scope::new(ItemData {
                        parent_id: Some(scope_id),
                        id: segment.clone().into(),
                        ..parent_scope_data
                    }));
                    self.get_mut(scope_id)
                        .scopes
                        .insert(segment.clone(), new_id);
                    new_id
                }
            };
        }
        self.get_mut(scope_id)
    }

    /// Gets the fully qualified display path of an item.
    pub fn fully_qualified_display_path_impl<T: Item>(
        &self,
        item: &T,
        separator: &str,
        (type_prefix, source_suffix): (bool, bool),
        relative_to: Option<ItemId<Scope>>,
        item_prefix: Option<&str>,
    ) -> String {
        let data = item.data();

        let mut path = vec![format!(
            "{}{}",
            item_prefix.unwrap_or_default(),
            data.id.as_str()
        )];
        let mut parent_id = data.parent_id;
        while let Some(this_parent_id) = parent_id {
            if let Some(relative_to) = relative_to {
                if this_parent_id == relative_to {
                    break;
                }
            }

            let parent = self.get(this_parent_id);
            let id = parent.data().id.to_string();
            if !id.is_empty() {
                path.push(id);
            }
            parent_id = parent.data().parent_id;
        }
        path.reverse();

        let prefix = if type_prefix {
            format!("{}:", T::TYPE.to_string().to_lowercase())
        } else {
            "".to_string()
        };
        format!(
            "{}{}{}",
            prefix,
            path.join(separator),
            if source_suffix {
                format!(" ({:?})", data.source)
            } else {
                "".to_string()
            }
        )
    }

    pub fn fully_qualified_display_path<T: Item>(
        &self,
        item: &T,
        relative_to: Option<ItemId<Scope>>,
        item_prefix: Option<&str>,
    ) -> String {
        self.fully_qualified_display_path_impl(item, "::", (false, false), relative_to, item_prefix)
    }

    /// Returns a topological sort of `id` and its dependencies.
    pub fn scope_and_dependencies(&self, id: ItemId<Package>) -> Vec<ItemId<Package>> {
        impl TopologicalSortable<ItemMap> for ItemId<Package> {
            fn dependencies(&self, items: &ItemMap) -> Vec<Self> {
                let item = items.get(*self);
                item.dependencies.values().map(|d| d.id).collect()
            }

            fn id(&self, items: &ItemMap) -> String {
                items.get(*self).data.id.to_string()
            }
        }

        topological_sort(std::iter::once(id), self).unwrap()
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ItemType {
    Component,
    Concept,
    Message,
    Type,
    Attribute,
    Scope,
    Package,
}
impl Display for ItemType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum ItemValue {
    Component(Component),
    Concept(Concept),
    Message(Message),
    Type(Type),
    Attribute(Attribute),
    Scope(Scope),
    Package(Package),
}

#[derive(Clone, PartialEq, Debug, Eq)]
pub struct ItemData {
    /// The parent scope of this item, if available
    pub parent_id: Option<ItemId<Scope>>,
    /// The identifier of this item
    pub id: Identifier,
    /// Where this item came from. Used to guide the code generation process.
    pub source: ItemSource,
}

#[derive(Copy, Clone, PartialEq, Debug, Eq)]
pub enum ItemSource {
    /// This is an item defined by the semantic system that should be present in all languages.
    /// Example: `String`, etc.
    System,
    /// This is an item defined by the Ambient API.
    /// Example: `Layout`, etc.
    Ambient,
    /// This is an item defined by the user.
    User,
}

pub trait Item: Clone {
    const TYPE: ItemType;
    type Unresolved: Eq + Debug;

    fn from_item_value(value: &ItemValue) -> Option<&Self>;
    fn from_item_value_mut(value: &mut ItemValue) -> Option<&mut Self>;
    fn into_item_value(self) -> ItemValue;

    fn data(&self) -> &ItemData;
}

/// This item supports being resolved by cloning.
pub(crate) trait ResolveClone: Item {
    fn resolve_clone(
        self,
        items: &mut ItemMap,
        context: &Context,
        definitions: &StandardDefinitions,
        self_id: ItemId<Self>,
    ) -> anyhow::Result<Self>;
}

pub struct ItemId<T: Item>(Ulid, PhantomData<T>);
impl<T: Item> std::hash::Hash for ItemId<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
    }
}
impl<T: Item> Copy for ItemId<T> {}
impl<T: Item> Clone for ItemId<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: Item> Debug for ItemId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ItemId<{}>({:?})", std::any::type_name::<T>(), self.0)
    }
}
impl<T: Item> Display for ItemId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}
impl<T: Item> PartialEq for ItemId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<T: Item> Eq for ItemId<T> {}
impl<T: Item> ItemId<T> {
    pub(crate) fn empty_you_should_really_initialize_this() -> Self {
        Self(Ulid::default(), PhantomData)
    }
}

#[derive(Clone)]
pub enum ResolvableItemId<T: Item> {
    Unresolved(T::Unresolved),
    Resolved(ItemId<T>),
}
impl<T: Item> ResolvableItemId<T> {
    pub fn as_resolved(&self) -> Option<ItemId<T>> {
        match self {
            Self::Unresolved(_) => None,
            Self::Resolved(id) => Some(*id),
        }
    }
}
impl<T: Item + Debug> Debug for ResolvableItemId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unresolved(arg0) => write!(f, "Unresolved({arg0:?})"),
            Self::Resolved(arg0) => write!(f, "Resolved({arg0:?})"),
        }
    }
}
impl<T: Item> PartialEq for ResolvableItemId<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Unresolved(l0), Self::Unresolved(r0)) => l0 == r0,
            (Self::Resolved(l0), Self::Resolved(r0)) => l0 == r0,
            _ => false,
        }
    }
}
impl<T: Item> Eq for ResolvableItemId<T> {}
impl<T: Item> std::hash::Hash for ResolvableItemId<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}
