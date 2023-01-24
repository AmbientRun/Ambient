use std::{
    any::{Any, TypeId}, collections::HashMap, fmt::Debug
};

use downcast_rs::{impl_downcast, Downcast};
use serde::{Deserialize, Serialize};

use crate::{Component, ComponentDesc, ComponentEntry, ComponentValue};

/// Represents a single attribute attached to a component
pub trait ComponentAttribute: 'static + Send + Sync + Downcast {}
impl_downcast!(ComponentAttribute);

#[derive(Default)]
pub struct AttributeStore {
    inner: HashMap<TypeId, Box<dyn ComponentAttribute>>,
}

impl AttributeStore {
    pub fn new() -> Self {
        Self { inner: Default::default() }
    }

    pub fn set<A: ComponentAttribute>(&mut self, attribute: A) {
        self.inner.insert(TypeId::of::<A>(), Box::new(attribute));
    }

    pub fn get_dyn(&self, key: TypeId) -> Option<&dyn ComponentAttribute> {
        self.inner.get(&key).map(|v| v.as_ref())
    }

    pub fn get<A: ComponentAttribute>(&self) -> Option<&A> {
        self.inner.get(&TypeId::of::<A>()).map(|v| v.downcast_ref::<A>().expect("Invalid type"))
    }

    /// Appends all attributes from `other` into self
    pub fn append(&mut self, other: Self) {
        self.inner.extend(other.inner)
    }
}

impl FromIterator<Box<dyn ComponentAttribute>> for AttributeStore {
    fn from_iter<T: IntoIterator<Item = Box<dyn ComponentAttribute>>>(iter: T) -> Self {
        Self { inner: iter.into_iter().map(|v| (v.as_ref().type_id(), v)).collect() }
    }
}

macro_rules! component_attributes {
    ($($name: ident,)*) => {
        $(
        impl $crate::ComponentAttribute for $name { }

        )*
    };
}

/// Initializes the attribute
pub trait AttributeConstructor<T, P>: 'static + Send + Sync {
    /// Construct a new instance of the attribute value and push it to the store
    fn construct(store: &mut AttributeStore, component: Component<T>, params: P);
}

#[derive(Clone, Copy)]
/// Declares a component as [`serde::Serialize`] and [`serde::Deserialize`]
///
/// Prefer [`Store`] or [`Networked`] rather than using directly
pub struct Serializable {
    ser: fn(&ComponentEntry) -> &dyn erased_serde::Serialize,
    deser: fn(ComponentDesc, &mut dyn erased_serde::Deserializer) -> Result<ComponentEntry, erased_serde::Error>,
    desc: ComponentDesc,
}

impl<T> AttributeConstructor<T, ()> for Serializable
where
    T: ComponentValue + Serialize + for<'de> Deserialize<'de>,
{
    fn construct(store: &mut AttributeStore, component: Component<T>, _: ()) {
        store.set(Self {
            ser: |v| v.downcast_ref::<T>() as &dyn erased_serde::Serialize,
            deser: |desc, deserializer| {
                let value = T::deserialize(deserializer)?;
                let entry = ComponentEntry::from_raw_parts(desc, value);
                Ok(entry)
            },
            desc: component.desc(),
        });
    }
}

impl Serializable {
    /// Serialize a value
    pub fn serialize<'a>(&self, entry: &'a ComponentEntry) -> &'a dyn erased_serde::Serialize {
        (self.ser)(entry)
    }
}

impl<'de> serde::de::DeserializeSeed<'de> for Serializable {
    type Value = ComponentEntry;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut deserializer = <dyn erased_serde::Deserializer>::erase(deserializer);
        let deserializer = &mut deserializer;
        (self.deser)(self.desc, deserializer).map_err(serde::de::Error::custom)
    }
}

pub struct Debuggable {
    debug: fn(&dyn Any) -> &dyn Debug,
}

impl Debuggable {
    pub(crate) fn as_debug<'a>(&self, value: &'a dyn Any) -> &'a dyn Debug {
        (self.debug)(value)
    }
}

impl<T> AttributeConstructor<T, ()> for Debuggable
where
    T: Debug,
{
    fn construct(store: &mut AttributeStore, _: Component<T>, _: ()) {
        store.set(Self { debug: |entry| entry.downcast_ref::<T>().unwrap() as &dyn Debug })
    }
}

/// Allows constructing a default value of the type
pub struct MakeDefault {
    make_default: Box<dyn Fn() -> ComponentEntry + Send + Sync>,
}

impl MakeDefault {
    /// Construct the default value of this component
    pub fn make_default(&self) -> ComponentEntry {
        (self.make_default)()
    }
}

impl<T: ComponentValue + Default> AttributeConstructor<T, ()> for MakeDefault {
    fn construct(store: &mut AttributeStore, component: Component<T>, _: ()) {
        store.set(Self { make_default: Box::new(move || ComponentEntry::new(component, T::default())) })
    }
}

impl<T: ComponentValue, F: 'static + Send + Sync + Fn() -> T> AttributeConstructor<T, F> for MakeDefault {
    fn construct(store: &mut AttributeStore, component: Component<T>, func: F) {
        store.set(Self { make_default: Box::new(move || ComponentEntry::new(component, func())) })
    }
}

/// Store the component on disc
///
/// Provides `Serializable`
#[derive(Debug, Clone, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Store;
/// Synchronize the component over the network to the clients
///
/// Provides `Serializable`
#[derive(Debug, Clone, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Networked;

impl<T> AttributeConstructor<T, ()> for Networked
where
    T: ComponentValue + Serialize + for<'de> Deserialize<'de>,
{
    fn construct(store: &mut AttributeStore, component: Component<T>, params: ()) {
        Serializable::construct(store, component, params);
        store.set(Self);
    }
}

impl<T> AttributeConstructor<T, ()> for Store
where
    T: ComponentValue + Serialize + for<'de> Deserialize<'de>,
{
    fn construct(store: &mut AttributeStore, component: Component<T>, params: ()) {
        Serializable::construct(store, component, params);
        store.set(Self);
    }
}

pub(crate) struct ComponentPath(pub String);

component_attributes! {
    Serializable,
    Debuggable,
    MakeDefault,
    Store,
    Networked,
    ComponentPath,
}
