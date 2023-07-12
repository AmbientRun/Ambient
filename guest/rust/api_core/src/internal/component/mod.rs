use crate::internal::wit;
use std::marker::PhantomData;

pub(crate) mod query;
pub(crate) mod traits;

mod entity;
pub use entity::*;

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
