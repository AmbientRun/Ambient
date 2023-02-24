use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    sync::Arc,
};

use downcast_rs::{impl_downcast, Downcast};
use serde::{Deserialize, Serialize};

use crate::{ComponentDesc, ComponentEntry, ComponentValue};

/// Represents a single attribute attached to a component
pub trait ComponentAttribute: 'static + Send + Sync + Downcast {
    fn type_name(&self) -> &'static str {
        type_name::<Self>()
    }
}

impl_downcast!(ComponentAttribute);

#[derive(Default, Clone)]
pub struct AttributeStore {
    inner: HashMap<TypeId, Arc<dyn ComponentAttribute>>,
}

impl AttributeStore {
    pub fn new() -> Self {
        Self { inner: Default::default() }
    }

    pub fn set<A: ComponentAttribute>(&mut self, attribute: A) {
        self.inner.insert(TypeId::of::<A>(), Arc::new(attribute));
    }

    pub fn set_dyn(&mut self, attribute: Arc<dyn ComponentAttribute>) {
        self.inner.insert((*attribute).type_id(), attribute);
    }

    pub fn get_dyn(&self, key: TypeId) -> Option<&dyn ComponentAttribute> {
        self.inner.get(&key).map(|v| v.as_ref())
    }

    pub fn get<A: ComponentAttribute>(&self) -> Option<&A> {
        self.inner.get(&TypeId::of::<A>()).map(|v| v.downcast_ref::<A>().expect("Invalid type"))
    }

    pub fn has<A: ComponentAttribute>(&self) -> bool {
        self.inner.contains_key(&TypeId::of::<A>())
    }

    /// Appends all attributes from `other` into self by cloning
    pub fn append(&mut self, other: &Self) {
        self.inner.extend(other.inner.iter().map(|(&k, v)| (k, v.clone())))
    }

    pub fn iter(&self) -> impl Iterator<Item = (TypeId, &Arc<dyn ComponentAttribute>)> {
        self.inner.iter().map(|(&k, v)| (k, v))
    }
}

impl Debug for AttributeStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_map();
        for (k, v) in &self.inner {
            s.entry(k, &v.type_name());
        }

        s.finish()
    }
}

impl FromIterator<Arc<dyn ComponentAttribute>> for AttributeStore {
    fn from_iter<T: IntoIterator<Item = Arc<dyn ComponentAttribute>>>(iter: T) -> Self {
        Self { inner: iter.into_iter().map(|v| ((*v).type_id(), v)).collect() }
    }
}

/// Initializes the attribute
pub trait AttributeConstructor<T, P>: 'static + Send + Sync {
    /// Construct a new instance of the attribute value and push it to the store
    fn construct(store: &mut AttributeStore, params: P);
}

#[derive(Clone, Copy)]
/// Declares a component as [`serde::Serialize`] and [`serde::Deserialize`]
///
/// Prefer [`Store`] or [`Networked`] rather than using directly
pub struct Serializable {
    ser: fn(&ComponentEntry) -> &dyn erased_serde::Serialize,
    deser: fn(ComponentDesc, &mut dyn erased_serde::Deserializer) -> Result<ComponentEntry, erased_serde::Error>,
}
impl ComponentAttribute for Serializable {}
impl<T> AttributeConstructor<T, ()> for Serializable
where
    T: ComponentValue + Serialize + for<'de> Deserialize<'de>,
{
    fn construct(store: &mut AttributeStore, _: ()) {
        store.set(Self {
            ser: |v| v.downcast_ref::<T>() as &dyn erased_serde::Serialize,
            deser: |desc, deserializer| {
                let value = T::deserialize(deserializer)?;
                let entry = ComponentEntry::from_raw_parts(desc, value);
                Ok(entry)
            },
        });
    }
}
impl Serializable {
    /// Serialize a value
    pub fn serialize<'a>(&self, entry: &'a ComponentEntry) -> &'a dyn erased_serde::Serialize {
        (self.ser)(entry)
    }

    /// Deserialize a value
    pub fn deserializer(&self, desc: ComponentDesc) -> ComponentDeserializer {
        ComponentDeserializer { desc, deser: self.deser }
    }
}

pub struct ComponentDeserializer {
    desc: ComponentDesc,
    deser: fn(ComponentDesc, &mut dyn erased_serde::Deserializer) -> Result<ComponentEntry, erased_serde::Error>,
}

impl<'de> serde::de::DeserializeSeed<'de> for ComponentDeserializer {
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
impl ComponentAttribute for Debuggable {}
impl Debuggable {
    pub(crate) fn as_debug<'a>(&self, value: &'a dyn Any) -> &'a dyn Debug {
        (self.debug)(value)
    }
}
impl<T> AttributeConstructor<T, ()> for Debuggable
where
    T: 'static + Debug,
{
    fn construct(store: &mut AttributeStore, _: ()) {
        store.set(Self { debug: |entry| entry.downcast_ref::<T>().unwrap() as &dyn Debug })
    }
}

/// Allows constructing a default value of the type
#[derive(Clone)]
pub struct MakeDefault {
    make_default: Arc<dyn Fn(ComponentDesc) -> ComponentEntry + Send + Sync>,
}
impl ComponentAttribute for MakeDefault {}
impl MakeDefault {
    /// Construct the default value of this component
    pub fn make_default(&self, desc: ComponentDesc) -> ComponentEntry {
        (self.make_default)(desc)
    }
}
impl<T: ComponentValue + Default> AttributeConstructor<T, ()> for MakeDefault {
    fn construct(store: &mut AttributeStore, _: ()) {
        store.set(Self { make_default: Arc::new(move |desc| ComponentEntry::from_raw_parts(desc, T::default())) })
    }
}
impl<T: ComponentValue, F: 'static + Send + Sync + Fn() -> T> AttributeConstructor<T, F> for MakeDefault {
    fn construct(store: &mut AttributeStore, func: F) {
        store.set(Self { make_default: Arc::new(move |desc| ComponentEntry::from_raw_parts(desc, func())) })
    }
}

/// Store the component on disc
///
/// Provides `Serializable`
#[derive(Debug, Clone, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Store;
impl ComponentAttribute for Store {}
impl<T> AttributeConstructor<T, ()> for Store
where
    T: ComponentValue + Serialize + for<'de> Deserialize<'de>,
{
    fn construct(store: &mut AttributeStore, params: ()) {
        <Serializable as AttributeConstructor<T, ()>>::construct(store, params);
        store.set(Self);
    }
}

/// Synchronize the component over the network to the clients
///
/// Provides `Serializable`
#[derive(Debug, Clone, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Networked;
impl ComponentAttribute for Networked {}
impl<T> AttributeConstructor<T, ()> for Networked
where
    T: ComponentValue + Serialize + for<'de> Deserialize<'de>,
{
    fn construct(store: &mut AttributeStore, params: ()) {
        <Serializable as AttributeConstructor<T, ()>>::construct(store, params);
        store.set(Self);
    }
}

pub(crate) struct ComponentPath(pub String);
impl ComponentAttribute for ComponentPath {}

/// A user-friendly name annotation, as opposed to the ID. (e.g. "Player Health" vs "player_health").
#[derive(Clone)]
pub struct Name(pub String);
impl ComponentAttribute for Name {}
impl<T: ComponentValue> AttributeConstructor<T, &str> for Name {
    fn construct(store: &mut AttributeStore, value: &str) {
        store.set(Self(value.to_string()))
    }
}

/// A user-friendly description. (e.g. "The player's health from 0 to 1.")
#[derive(Clone)]
pub struct Description(pub String);
impl ComponentAttribute for Description {}
impl<T: ComponentValue> AttributeConstructor<T, &str> for Description {
    fn construct(store: &mut AttributeStore, value: &str) {
        store.set(Self(value.to_string()))
    }
}

/// Indicates that this component was externally added.
#[derive(Clone)]
pub struct External;
impl ComponentAttribute for External {}
impl<T: ComponentValue> AttributeConstructor<T, ()> for External {
    fn construct(store: &mut AttributeStore, _: ()) {
        store.set(Self)
    }
}

/// Indicates that this component should be used as a resource only.
#[derive(Clone)]
pub struct Resource;
impl ComponentAttribute for Resource {}
impl<T: ComponentValue> AttributeConstructor<T, ()> for Resource {
    fn construct(store: &mut AttributeStore, _: ()) {
        store.set(Self)
    }
}

/// Indicates that this component can be used as a resource or component.
///
/// Typically used on components that are at the root of a Prefab and attached as resources.
#[derive(Clone)]
pub struct MaybeResource;
impl ComponentAttribute for MaybeResource {}
impl<T: ComponentValue> AttributeConstructor<T, ()> for MaybeResource {
    fn construct(store: &mut AttributeStore, _: ()) {
        store.set(Self)
    }
}

/// Provides a default for this component that will work with `MakeDefault`.
#[derive(Clone)]
pub struct DefaultValue<T: ComponentValue>(pub T);
impl<T: ComponentValue> ComponentAttribute for DefaultValue<T> {}
impl<T: ComponentValue> AttributeConstructor<T, T> for DefaultValue<T> {
    fn construct(store: &mut AttributeStore, value: T) {
        store.set(Self(value.clone()));
        MakeDefault::construct(store, move || value.clone());
    }
}
