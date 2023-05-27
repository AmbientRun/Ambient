use ambient_project::Identifier;
use ulid::Ulid;

use crate::{Attribute, Component, Concept, Context, Message, Scope, Type};
use anyhow::Context as AnyhowContext;
use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    fmt::{Debug, Display},
    marker::PhantomData,
};

#[derive(Clone, PartialEq, Debug, Default)]
pub struct ItemMap {
    items: HashMap<Ulid, RefCell<ItemValue>>,
    vec_items: HashMap<ItemId<Type>, ItemId<Type>>,
    option_items: HashMap<ItemId<Type>, ItemId<Type>>,
}
impl ItemMap {
    pub fn add<T: Item>(&mut self, item: T) -> ItemId<T> {
        let value = item.into_item_value();
        let is_type = matches!(value, ItemValue::Type(_));

        let new_id = self.add_raw(value);

        if is_type {
            let new_id = ItemId::<Type>(new_id.0, PhantomData);

            let vec_id = self.add_raw(Type::Vec(new_id).into_item_value());
            self.vec_items.insert(new_id, vec_id);

            let option_id = self.add_raw(Type::Option(new_id).into_item_value());
            self.option_items.insert(new_id, option_id);
        }

        new_id
    }

    fn add_raw<T: Item>(&mut self, value: ItemValue) -> ItemId<T> {
        let ulid = ulid::Ulid::new();
        self.items.insert(ulid, RefCell::new(value));
        ItemId(ulid, PhantomData)
    }

    /// Returns a reference to the item with the given id.
    ///
    /// Does not resolve the item.
    pub fn get<T: Item>(&self, id: ItemId<T>) -> anyhow::Result<Ref<T>> {
        let value = self
            .items
            .get(&id.0)
            .with_context(|| format!("Item not found: {id}"))?;

        Ok(Ref::map(value.borrow(), |r| T::from_item_value(r).unwrap()))
    }

    /// Returns a mutable reference to the item with the given id.
    ///
    /// Does not resolve the item.
    pub fn get_mut<T: Item>(&self, id: ItemId<T>) -> anyhow::Result<RefMut<T>> {
        let value = self
            .items
            .get(&id.0)
            .with_context(|| format!("Item not found: {id}"))?;

        Ok(RefMut::map(value.borrow_mut(), |r| {
            T::from_item_value_mut(r).unwrap()
        }))
    }

    pub fn insert<T: Item>(&mut self, id: ItemId<T>, item: T) {
        self.items
            .insert(id.0, RefCell::new(item.into_item_value()));
    }

    /// Resolve the item with the given id in-place, and return a mutable reference to it.
    pub(crate) fn resolve<T: Resolve>(
        &self,
        id: ItemId<T>,
        context: &Context,
    ) -> anyhow::Result<RefMut<T>> {
        let mut item = self.get_mut(id)?;
        item.resolve(self, id, context)?;
        Ok(item)
    }

    /// Resolve the item with the given id by cloning it, avoiding borrowing issues.
    pub(crate) fn resolve_clone<T: ResolveClone>(
        &mut self,
        id: ItemId<T>,
        context: &Context,
    ) -> anyhow::Result<()> {
        let item = self.get(id)?.clone();
        let new_item = item.resolve_clone(self, id, context)?;
        self.insert(id, new_item);
        Ok(())
    }

    pub fn get_vec_id(&self, id: ItemId<Type>) -> ItemId<Type> {
        self.vec_items.get(&id).copied().unwrap()
    }

    pub fn get_option_id(&self, id: ItemId<Type>) -> ItemId<Type> {
        self.option_items.get(&id).copied().unwrap()
    }

    pub(crate) fn get_scope<'a>(
        &'a self,
        start_scope_id: ItemId<Scope>,
        path: &[Identifier],
    ) -> anyhow::Result<Ref<Scope>> {
        let mut scope_id = start_scope_id;
        for segment in path.iter() {
            let scope = self.get(scope_id)?;
            scope_id = *scope
                .scopes
                .get(segment)
                .with_context(|| format!("failed to find scope {segment} in {scope_id}",))?;
        }
        self.get(scope_id)
    }

    pub(crate) fn get_or_create_scope_mut<'a>(
        &'a mut self,
        start_scope_id: ItemId<Scope>,
        path: &[Identifier],
    ) -> anyhow::Result<RefMut<Scope>> {
        let mut scope_id = start_scope_id;
        for segment in path.iter() {
            let existing_id = self.get(scope_id)?.scopes.get(segment).copied();
            scope_id = match existing_id {
                Some(id) => id,
                None => {
                    let new_id = self.add(Scope {
                        id: segment.clone(),
                        scopes: Default::default(),
                        components: Default::default(),
                        concepts: Default::default(),
                        messages: Default::default(),
                        types: Default::default(),
                        attributes: Default::default(),
                    });
                    self.get_mut(scope_id)?
                        .scopes
                        .insert(segment.clone(), new_id);
                    new_id
                }
            };
        }
        self.get_mut(scope_id)
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
}

#[derive(Clone, PartialEq, Debug)]
pub enum ItemValue {
    Component(Component),
    Concept(Concept),
    Message(Message),
    Type(Type),
    Attribute(Attribute),
    Scope(Scope),
}

pub trait Item: Clone {
    const TYPE: ItemType;
    type Unresolved: Eq + Debug;

    fn from_item_value(value: &ItemValue) -> Option<&Self>;
    fn from_item_value_mut(value: &mut ItemValue) -> Option<&mut Self>;
    fn into_item_value(self) -> ItemValue;
}

/// This item supports being resolved in-place.
pub(crate) trait Resolve: Item {
    fn resolve(
        &mut self,
        items: &ItemMap,
        self_id: ItemId<Self>,
        context: &Context,
    ) -> anyhow::Result<()>;
}

/// This item supports being resolved by cloning.
pub(crate) trait ResolveClone: Item {
    fn resolve_clone(
        self,
        items: &mut ItemMap,
        self_id: ItemId<Self>,
        context: &Context,
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

#[derive(Clone)]
pub enum ResolvableItemId<T: Item> {
    Unresolved(T::Unresolved),
    Resolved(ItemId<T>),
}
impl<T: Item> ResolvableItemId<T> {
    pub(crate) fn as_resolved(&self) -> Option<ItemId<T>> {
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
