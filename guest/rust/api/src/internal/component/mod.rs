use crate::internal::wit;
use std::{collections::HashMap, marker::PhantomData};

pub(crate) mod query;
pub(crate) mod traits;

pub use traits::{get_component as __internal_get_component, SupportedValue};

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

/// An [Entity] is a collection of components and associated values.
///
/// Use the [spawn](Entity::spawn) method to insert the [Entity] into the world.
#[derive(Clone, Default)]
pub struct Entity(pub(crate) HashMap<u32, wit::component::Value>);
impl Entity {
    /// Creates a new `Entity`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if this has `component`.
    pub fn has<T: SupportedValue>(&self, component: Component<T>) -> bool {
        self.0.contains_key(&component.index())
    }

    /// Gets the data for `component` in this, if it exists.
    pub fn get<T: SupportedValue>(&self, component: Component<T>) -> Option<T> {
        T::from_result(self.0.get(&component.index())?.clone())
    }

    /// TODO: Temporary fix to get UI working, as UI requires get_ref for String components, which exists for the native Entity
    #[doc(hidden)]
    pub fn get_ref<T: SupportedValue>(&self, component: Component<T>) -> Option<T> {
        T::from_result(self.0.get(&component.index())?.clone())
    }

    /// Adds `component` to this with `value`. It will replace an existing component if present.
    pub fn set<T: SupportedValue>(&mut self, component: Component<T>, value: T) {
        self.0.insert(component.index(), value.into_result());
    }

    /// Sets the `component` in this to the default value for `T`.
    pub fn set_default<T: SupportedValue + Default>(&mut self, component: Component<T>) {
        self.set(component, T::default())
    }

    /// Adds `component` to this with `value`, and returns `self` to allow for easy chaining.
    pub fn with<T: SupportedValue>(mut self, component: Component<T>, value: T) -> Self {
        self.set(component, value);
        self
    }

    /// Sets the `component` in this to the default value for `T`, and returns `self` to allow for easy chaining.
    pub fn with_default<T: SupportedValue + Default>(mut self, component: Component<T>) -> Self {
        self.set_default(component);
        self
    }

    /// Merges in the `other` Entity and returns this; any fields that were present in both will be replaced by `other`'s.
    pub fn with_merge(mut self, other: Entity) -> Self {
        self.merge(other);
        self
    }

    /// Removes the specified component from this, and returns the value if it was present.
    pub fn remove<T: SupportedValue>(&mut self, component: Component<T>) -> Option<T> {
        T::from_result(self.0.remove(&component.index())?)
    }

    /// Merges in the `other` Entity; any fields that were present in both will be replaced by `other`'s.
    pub fn merge(&mut self, other: Entity) {
        self.0.extend(other.0.into_iter());
    }

    /// Spawns an entity with these components.
    ///
    /// Returns `spawned_entity_uid`.
    pub fn spawn(&self) -> crate::prelude::EntityId {
        crate::entity::spawn(self)
    }

    pub(crate) fn call_with<R>(
        &self,
        callback: impl FnOnce(&[(u32, &wit::component::Value)]) -> R,
    ) -> R {
        let data = self
            .0
            .iter()
            .map(|(idx, val)| (*idx, val))
            .collect::<Vec<_>>();
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
    fn from_component_types(component_types: Vec<wit::component::Value>) -> Option<Self::Data>;
}

// From: https://stackoverflow.com/questions/56697029/is-there-a-way-to-impl-trait-for-a-tuple-that-may-have-any-number-elements
macro_rules! tuple_impls {
    ( $( $name:ident )+ ) => {
        impl<$($name: SupportedValue),+> ComponentsTuple for ($(Component<$name>,)+) {
            #[allow(unused_parens)]
            type Data = ($($name),+);

            fn as_indices(&self) -> Vec<u32> {
                #[allow(non_snake_case)]
                let ($($name,)+) = self;
                vec![$($name.index(),)*]
            }
            fn from_component_types(component_types: Vec<wit::component::Value>) -> Option<Self::Data> {
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
impl<T: SupportedValue> ComponentsTuple for Component<T> {
    type Data = T;

    fn as_indices(&self) -> Vec<u32> {
        vec![self.index()]
    }
    fn from_component_types(component_types: Vec<wit::component::Value>) -> Option<Self::Data> {
        assert_eq!(component_types.len(), 1);
        T::from_result(component_types[0].clone())
    }
}
impl ComponentsTuple for () {
    type Data = ();

    fn as_indices(&self) -> Vec<u32> {
        vec![]
    }
    fn from_component_types(component_types: Vec<wit::component::Value>) -> Option<Self::Data> {
        assert!(component_types.is_empty());
        Some(())
    }
}
