use ulid::Ulid;

use crate::{Attribute, Component, Concept, Context, Message, Scope, Type};
use anyhow::Context as AnyhowContext;
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    marker::PhantomData,
};

#[derive(Clone, PartialEq, Debug, Default)]
pub struct ItemMap {
    items: HashMap<Ulid, ItemValue>,
    vec_items: HashMap<ItemId<Type>, ItemId<Type>>,
    option_items: HashMap<ItemId<Type>, ItemId<Type>>,
}
impl ItemMap {
    pub fn add<T: Item>(&mut self, item: T) -> ItemId<T> {
        let ulid = ulid::Ulid::new();
        self.items.insert(ulid, item.into_item_value());
        ItemId(ulid, PhantomData)
    }

    pub fn get_without_resolve<T: Item>(&self, id: ItemId<T>) -> anyhow::Result<&T> {
        T::from_item_value(
            self.items
                .get(&id.0)
                .with_context(|| format!("Item not found: {id}"))?,
        )
        .with_context(|| format!("Item is the wrong type: {id}"))
    }

    pub fn get_mut<T: Item>(&mut self, id: ItemId<T>) -> anyhow::Result<&mut T> {
        T::from_item_value_mut(
            self.items
                .get_mut(&id.0)
                .with_context(|| format!("Item not found: {id}"))?,
        )
        .with_context(|| format!("Item is the wrong type: {id}"))
    }

    pub fn insert<T: Item>(&mut self, id: ItemId<T>, item: T) {
        self.items.insert(id.0, item.into_item_value());
    }

    pub fn resolve<T: Item>(&mut self, id: ItemId<T>, context: &Context) -> anyhow::Result<&mut T> {
        let item = self
            .get_without_resolve(id)?
            .clone()
            .resolve(self, id, context)?;
        self.insert(id, item);
        self.get_mut(id)
    }

    pub fn get_vec_id(&mut self, id: ItemId<Type>) -> ItemId<Type> {
        if let Some(id) = self.vec_items.get(&id).cloned() {
            return id;
        }

        let vec_id = self.add(Type::Vec(id));
        self.vec_items.insert(id, vec_id);

        vec_id
    }

    pub fn get_option_id(&mut self, id: ItemId<Type>) -> ItemId<Type> {
        if let Some(id) = self.option_items.get(&id).cloned() {
            return id;
        }

        let option_id = self.add(Type::Option(id));
        self.option_items.insert(id, option_id);

        option_id
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
    fn resolve(
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
