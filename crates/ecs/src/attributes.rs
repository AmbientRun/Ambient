use std::{any::Any, fmt::Debug};

use serde::{Deserialize, Serialize};

use crate::{Component, ComponentDesc, ComponentEntry, ComponentValue};

pub trait ComponentAttribute: 'static + Send + Sync {}

macro_rules! component_attributes {
    ($($name: ident,)*) => {
$(
        impl $crate::ComponentAttribute for $name { }

)        *
    };
}

/// Declares an attribute type which can be attached to a component
pub trait ComponentAttributeConstructor<T, P>: 'static + Send + Sync {
    /// Construct a new instance of the attribute value
    fn construct(component: Component<T>, params: P) -> Self;
}

#[derive(Clone, Copy)]
/// Declares a component as [`serde::Serialize`] and [`serde::Deserialize`]
pub struct Serializable {
    ser: fn(&ComponentEntry) -> &dyn erased_serde::Serialize,
    deser: fn(ComponentDesc, &mut dyn erased_serde::Deserializer) -> Result<ComponentEntry, erased_serde::Error>,
    desc: ComponentDesc,
}

impl<T: ComponentValue + Serialize + for<'de> Deserialize<'de>> ComponentAttributeConstructor<T, ()> for Serializable {
    fn construct(component: Component<T>, _: ()) -> Self {
        Self {
            ser: |v| v.downcast_ref::<T>() as &dyn erased_serde::Serialize,
            deser: |desc, deserializer| {
                let value = T::deserialize(deserializer)?;
                let entry = ComponentEntry::from_raw_parts(desc, value);
                Ok(entry)
            },
            desc: component.desc(),
        }
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

impl<T> ComponentAttributeConstructor<T, ()> for Debuggable
where
    T: Debug,
{
    fn construct(_: Component<T>, _: ()) -> Self {
        Self { debug: |entry| entry.downcast_ref::<T>().unwrap() as &dyn Debug }
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

impl<T: ComponentValue + Default> ComponentAttributeConstructor<T, ()> for MakeDefault {
    fn construct(component: Component<T>, _: ()) -> Self {
        Self { make_default: Box::new(move || ComponentEntry::new(component, T::default())) }
    }
}

impl<T: ComponentValue, F: 'static + Send + Sync + Fn() -> T> ComponentAttributeConstructor<T, F> for MakeDefault {
    fn construct(component: Component<T>, func: F) -> Self {
        Self { make_default: Box::new(move || ComponentEntry::new(component, func())) }
    }
}

/// Store the component on disc
#[derive(Debug, Clone, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Store;
/// Synchronize the component over the network to the clients
#[derive(Debug, Clone, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Networked;

/// Automatically implement for marker types
impl<A, T> ComponentAttributeConstructor<T, ()> for A
where
    A: ComponentAttribute + Default,
{
    fn construct(_: Component<T>, _: ()) -> Self {
        Self::default()
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
