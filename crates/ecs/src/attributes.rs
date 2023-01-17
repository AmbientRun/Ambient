use std::{any::Any, fmt::Debug};

use serde::{Deserialize, Serialize};

use crate::{Component, ComponentDesc, ComponentEntry, ComponentValue};

/// Declares an attribute type which can be attached to a component
pub trait ComponentAttribute: 'static {
    type Value: 'static + Send + Sync;
}

macro_rules! component_attributes {
    ($($(#[$outer: meta])* $name: ident: $ty: ty,)*) => {
$(
        /// Component attribute
        $(#[$outer])*
        #[derive(Default, Eq, PartialEq, PartialOrd, Hash, Debug, Clone)]
        pub struct $name {}

        impl $crate::ComponentAttribute for $name {
            type Value = $ty;
        }

)        *
    };
}

pub trait ComponentAttributeValue<T, P> {
    /// Construct a new instance of the attribute value
    fn construct(component: Component<T>, params: P) -> Self;
}

/// Allow serializing a component entry
#[derive(Clone, Copy)]
pub struct ComponentSerializer {
    ser: fn(&ComponentEntry) -> &dyn erased_serde::Serialize,
    deser: fn(ComponentDesc, &mut dyn erased_serde::Deserializer) -> Result<ComponentEntry, erased_serde::Error>,
    desc: ComponentDesc,
}

impl<T: ComponentValue + Serialize + for<'de> Deserialize<'de>> ComponentAttributeValue<T, ()> for ComponentSerializer {
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

impl ComponentSerializer {
    /// Serialize a value
    pub fn serialize<'a>(&self, entry: &'a ComponentEntry) -> &'a dyn erased_serde::Serialize {
        (self.ser)(entry)
    }
}

impl<'de> serde::de::DeserializeSeed<'de> for ComponentSerializer {
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

pub struct ComponentDebugger {
    debug: fn(&dyn Any) -> &dyn Debug,
}

impl ComponentDebugger {
    pub(crate) fn as_debug<'a>(&self, value: &'a dyn Any) -> &'a dyn Debug {
        (self.debug)(value)
    }
}

impl<T> ComponentAttributeValue<T, ()> for ComponentDebugger
where
    T: Debug,
{
    fn construct(_: Component<T>, _: ()) -> Self {
        Self { debug: |entry| entry.downcast_ref::<T>().unwrap() as &dyn Debug }
    }
}

pub struct ComponentDefault {
    make_default: Box<dyn Fn() -> ComponentEntry + Send + Sync>,
}

impl ComponentDefault {
    /// Construct the default value of this component
    pub fn make_default(&self) -> ComponentEntry {
        (self.make_default)()
    }
}

impl<T: ComponentValue + Default> ComponentAttributeValue<T, ()> for ComponentDefault {
    fn construct(component: Component<T>, _: ()) -> Self {
        Self { make_default: Box::new(move || ComponentEntry::new(component, T::default())) }
    }
}

impl<T: ComponentValue, F: 'static + Send + Sync + Fn() -> T> ComponentAttributeValue<T, F> for ComponentDefault {
    fn construct(component: Component<T>, func: F) -> Self {
        Self { make_default: Box::new(move || ComponentEntry::new(component, func())) }
    }
}

component_attributes! {
    /// Declares a component as [`serde::Serialize`] and [`serde::Deserialize`]
    Serializable: ComponentSerializer,
    Debuggable: ComponentDebugger,
    MakeDefault: ComponentDefault,
    Store: (),
    Networked: (),
}

impl<T> ComponentAttributeValue<T, ()> for () {
    fn construct(_: Component<T>, _: ()) -> Self {}
}
