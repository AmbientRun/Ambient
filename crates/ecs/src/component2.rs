use std::{
    any::{Any, TypeId}, marker::PhantomData
};

use serde::Serialize;

use crate::{ComponentValue, ComponentValueBase};

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
    pub clone: fn(&dyn Any) -> Box<dyn ComponentValueBase>,
    pub construct_default: Option<fn() -> Box<dyn ComponentValueBase>>,
    pub serialize: Option<fn(&dyn Any) -> &dyn erased_serde::Serialize>,
    pub custom_attrs: &'static [(&'static str, &'static dyn Any)],
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn manual_component() {
        let vtable = &ComponentVTable {
            name: "my_component",
            type_id: || TypeId::of::<String>(),
            clone: |v| Box::new(v.downcast_ref::<String>().unwrap().clone()),
            construct_default: Some(|| Box::<String>::default() as Box<dyn ComponentValueBase>),
            serialize: Some(|v| v.downcast_ref::<String>().unwrap() as &dyn erased_serde::Serialize),
            custom_attrs: &[
                ("is_networked", &()),
                ("is_stored", &()),
                ("to_pretty", &|v: &dyn Any| format!("value: {}", v.downcast_ref::<String>().unwrap())),
            ],
        };

        let component: Component<String> = Component::new(1, vtable);
    }
}
