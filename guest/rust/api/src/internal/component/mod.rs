use crate::internal::host;
use std::{collections::HashMap, marker::PhantomData};

mod borrowed_types;
pub(crate) mod query;
pub(crate) mod traits;

pub use traits::{
    get_component as __internal_get_component, SupportedComponentTypeGet, SupportedComponentTypeSet,
};

/// Implemented by all [Component]s.
pub trait UntypedComponent {
    #[doc(hidden)]
    fn index(&self) -> u32;
}

/// A component (piece of entity data). See [entity::get_component](crate::entity::get_component) and [entity::set_component](crate::entity::set_component).
#[derive(Debug)]
pub struct Component<T> {
    index: u32,
    _phantom: PhantomData<T>,
}
impl<T> Clone for Component<T> {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            _phantom: PhantomData,
        }
    }
}
impl<T> Copy for Component<T> {}
impl<T> Component<T> {
    #[doc(hidden)]
    pub const fn new(index: u32) -> Self {
        Self {
            index,
            _phantom: PhantomData,
        }
    }
}
impl<T> UntypedComponent for Component<T> {
    fn index(&self) -> u32 {
        self.index
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! lazy_component {
    ($id:literal) => {
        $crate::LazyComponent::new(|| $crate::__internal_get_component($id))
    };
}

/// Contains a group of components and associated values.
///
/// These can be used to spawn entities.
#[derive(Clone)]
pub struct Components(pub(crate) HashMap<u32, host::ComponentTypeResult>);
impl Components {
    /// Creates a new `Components`.
    pub fn new() -> Self {
        Self(Default::default())
    }

    /// Gets the data for `component` in this, if it exists.
    pub fn get<T: SupportedComponentTypeGet>(&self, component: Component<T>) -> Option<T> {
        T::from_result(self.0.get(&component.index())?.clone())
    }

    /// Adds `component` to this with `value`. It will replace an existing component if present.
    pub fn set<T: SupportedComponentTypeSet>(&mut self, component: Component<T>, value: T) {
        self.0.insert(component.index(), value.into_result());
    }

    /// Sets the `component` in this to the default value for `T`.
    pub fn set_default<T: SupportedComponentTypeSet + Default>(&mut self, component: Component<T>) {
        self.set(component, T::default())
    }

    /// Adds `value` to this `EntityData`, and returns `self` to allow for easy chaining.
    pub fn with<T: SupportedComponentTypeSet>(mut self, component: Component<T>, value: T) -> Self {
        self.set(component, value);
        self
    }

    /// Adds the default value for `T` to this `EntityData`, and returns `self` to allow for easy chaining.
    pub fn with_default<T: SupportedComponentTypeSet + Default>(
        mut self,
        component: Component<T>,
    ) -> Self {
        self.set_default(component);
        self
    }

    /// Spawns an entity with these components. If `persistent` is set, this entity will not be
    /// removed when this module is unloaded.
    ///
    /// This is an asynchronous operation; use [entity::wait_for_spawn](crate::entity::wait_for_spawn) to get notified when
    /// the entity is spawned.
    ///
    /// Returns `spawned_entity_uid`.
    pub fn spawn(&self, persistent: bool) -> crate::prelude::EntityId {
        crate::entity::spawn(self, persistent)
    }

    pub(crate) fn call_with<R>(
        &self,
        callback: impl FnOnce(&[(u32, host::ComponentTypeParam<'_>)]) -> R,
    ) -> R {
        let data = borrowed_types::create_owned_types(&self.0);
        let data = borrowed_types::create_borrowed_types(&data);
        callback(&data)
    }
}

/// A tuple of [Component]s.
pub trait ComponentsTuple {
    /// The types of the data stored in this tuple
    type Data;

    #[doc(hidden)]
    fn as_indices(&self) -> Vec<u32>;
    #[doc(hidden)]
    fn from_component_types(component_types: Vec<host::ComponentTypeResult>) -> Option<Self::Data>;
}

// From: https://stackoverflow.com/questions/56697029/is-there-a-way-to-impl-trait-for-a-tuple-that-may-have-any-number-elements
macro_rules! tuple_impls {
    ( $( $name:ident )+ ) => {
        impl<$($name: SupportedComponentTypeGet),+> ComponentsTuple for ($(Component<$name>,)+) {
            #[allow(unused_parens)]
            type Data = ($($name),+);

            fn as_indices(&self) -> Vec<u32> {
                #[allow(non_snake_case)]
                let ($($name,)+) = self;
                vec![$($name.index(),)*]
            }
            fn from_component_types(component_types: Vec<host::ComponentTypeResult>) -> Option<Self::Data> {
                paste::paste! {
                    #[allow(non_snake_case)]
                    if let [$([<value_ $name>],)+] = &component_types[..] {
                        Some(($($name::from_result([<value_ $name>].clone())?),+))
                    } else {
                        None
                    }
                }
            }
        }
    };
}
tuple_impls! { A }
tuple_impls! { A B }
tuple_impls! { A B C }
tuple_impls! { A B C D }
tuple_impls! { A B C D E }
tuple_impls! { A B C D E F }
tuple_impls! { A B C D E F G }
tuple_impls! { A B C D E F G H }
tuple_impls! { A B C D E F G H I }
impl<T: SupportedComponentTypeGet> ComponentsTuple for Component<T> {
    type Data = T;

    fn as_indices(&self) -> Vec<u32> {
        vec![self.index()]
    }
    fn from_component_types(component_types: Vec<host::ComponentTypeResult>) -> Option<Self::Data> {
        assert_eq!(component_types.len(), 1);
        T::from_result(component_types[0].clone())
    }
}
impl ComponentsTuple for () {
    type Data = ();

    fn as_indices(&self) -> Vec<u32> {
        vec![]
    }
    fn from_component_types(component_types: Vec<host::ComponentTypeResult>) -> Option<Self::Data> {
        assert!(component_types.is_empty());
        Some(())
    }
}
