use std::{
    any::{type_name, Any, TypeId}, marker::PhantomData
};

use downcast_rs::Downcast;
use profiling::puffin_scope;
use serde::Serialize;

/// Defines an object safe trait which allows for downcasting
pub trait ComponentValue: 'static + Send + Sync {
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
    fn as_any(&self) -> &dyn Any;
    fn type_name(&self) -> &'static str;
}

impl<T: 'static + Send + Sync> ComponentValue for T {
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self as Box<dyn Any>
    }
    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn type_name(&self) -> &'static str {
        type_name::<Self>()
    }
}

impl dyn ComponentValue {
    fn try_downcast<T: 'static>(self: Box<Self>) -> Option<Box<T>> {
        self.into_any().downcast().ok()
    }

    fn try_downcast_ref<T: 'static>(&self) -> Option<&T> {
        self.as_any().downcast_ref()
    }

    pub fn downcast_ref<T: 'static>(&self) -> &T {
        #[cfg(debug_assertions)]
        match self.try_downcast_ref() {
            Some(v) => v,
            None => {
                panic!("Mismatched type. Expected: {:?}. Found {:?}", type_name::<T>(), self.type_name());
            }
        }
        #[cfg(not(debug_assertions))]
        self.try_downcast_ref().expect("Mismatched type")
    }

    pub fn downcast<T: 'static>(self: Box<Self>) -> Box<T> {
        #[cfg(debug_assertions)]
        let n = self.type_name();
        match self.try_downcast() {
            Some(v) => v,
            None => {
                panic!("Mismatched type. Expected: {:?}, found {n:?}", type_name::<T>());
            }
        }
        #[cfg(not(debug_assertions))]
        self.try_downcast().expect("Mismatched type")
    }
}

pub struct Attribute {
    key: &'static str,
    /// To use: cast to the correct type, which is determined by the attribute in use.
    ///
    /// It is recommended to use helper functions
    value: &'static dyn ComponentValue,
}

impl Attribute {
    pub const fn new(key: &'static str, value: &'static dyn ComponentValue) -> Self {
        Self { key, value }
    }
}

pub fn slice_attrs(attrs: &'static [Attribute], key: &str) -> Option<&'static Attribute> {
    attrs.iter().find(|v| v.key == key)
}

pub struct Component<T> {
    index: i32,
    vtable: &'static ComponentVTable,
    _marker: PhantomData<T>,
}

impl<T> Component<T> {
    fn new(index: i32, vtable: &'static ComponentVTable) -> Self {
        Self { index, vtable, _marker: PhantomData }
    }
}

/// Holds untyped information for everything a component can do
struct ComponentVTable {
    pub name: &'static str,
    pub type_id: fn() -> TypeId,
    pub clone: fn(&dyn ComponentValue) -> Box<dyn ComponentValue>,
    pub construct_default: Option<fn() -> Box<dyn ComponentValue>>,
    pub serialize: Option<fn(&dyn ComponentValue) -> &dyn erased_serde::Serialize>,
    pub custom_attrs: fn(&str) -> Option<&'static Attribute>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn manual_component() {
        static ATTRS: &[Attribute] = &[
            Attribute::new("is_networked", &()),
            Attribute::new("is_stored", &()),
            Attribute::new("display", &|v: &dyn Any| format!("value: {}", v.downcast_ref::<String>().unwrap())),
        ];

        static VTABLE: &ComponentVTable = &ComponentVTable {
            // let vtable: &'static ComponentVTable = &ComponentVTable {
            name: "my_component",
            type_id: || TypeId::of::<String>(),
            clone: |v| Box::new(v.downcast_ref::<String>().clone()),
            construct_default: Some(|| Box::<String>::default() as Box<dyn ComponentValue>),
            serialize: Some(|v| v.downcast_ref::<String>() as &dyn erased_serde::Serialize),
            custom_attrs: |key| slice_attrs(ATTRS, key),
        };

        let component: Component<String> = Component::new(1, VTABLE);

        let value = "Hello, World".to_string();

        let value_cloned = *(component.vtable.clone)(&value as _).downcast::<String>();

        assert_eq!(value, value_cloned);
    }
}
