use crate::internal::wit;
use std::marker::PhantomData;

pub(crate) mod query;
pub(crate) mod traits;

mod entity;
pub use entity::*;

pub use traits::{
    get_component as __internal_get_component, ComponentOptionValue, ComponentValue,
    ComponentVecValue, SupportedValue, SupportedValueRef,
};

pub(crate) use ambient_shared_types::ComponentIndex;

/// Implemented by all [Component]s.
pub trait UntypedComponent {
    #[doc(hidden)]
    fn index(&self) -> ComponentIndex;
}

/// A component (piece of entity data). See [entity::get_component](crate::entity::get_component) and [entity::set_component](crate::entity::set_component).
#[derive(Debug)]
pub struct Component<T> {
    index: ComponentIndex,
    _phantom: PhantomData<T>,
}
impl<T> Clone for Component<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for Component<T> {}
impl<T> Component<T> {
    #[doc(hidden)]
    pub const fn new(index: ComponentIndex) -> Self {
        Self {
            index,
            _phantom: PhantomData,
        }
    }
}
impl<T> UntypedComponent for Component<T> {
    fn index(&self) -> ComponentIndex {
        self.index
    }
}

/// A tuple of [Component]s.
pub trait ComponentsTuple {
    /// The types of the data stored in this tuple
    type Data;

    #[doc(hidden)]
    fn as_indices(&self) -> Vec<ComponentIndex>;
    #[doc(hidden)]
    fn from_component_types(component_types: Vec<wit::component::Value>) -> Option<Self::Data>;
}

// From: https://stackoverflow.com/questions/56697029/is-there-a-way-to-impl-trait-for-a-tuple-that-may-have-any-number-elements
macro_rules! tuple_impls {
    ( $( $name:ident )+ ) => {
        impl<$($name: SupportedValue),+> ComponentsTuple for ($(Component<$name>,)+) {
            #[allow(unused_parens)]
            type Data = ($($name,)+);

            fn as_indices(&self) -> Vec<ComponentIndex> {
                #[allow(non_snake_case)]
                let ($($name,)+) = self;
                vec![$($name.index(),)*]
            }
            fn from_component_types(component_types: Vec<wit::component::Value>) -> Option<Self::Data> {
                paste::paste! {
                    #[allow(non_snake_case)]
                    if let [$([<value_ $name>],)+] = &component_types[..] {
                        Some(($($name::from_result([<value_ $name>].clone())?,)+))
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
tuple_impls! { A B C D E F G H I J }
tuple_impls! { A B C D E F G H I J K }
tuple_impls! { A B C D E F G H I J K L }
tuple_impls! { A B C D E F G H I J K L M }
tuple_impls! { A B C D E F G H I J K L M N }
tuple_impls! { A B C D E F G H I J K L M N O }
tuple_impls! { A B C D E F G H I J K L M N O P }
tuple_impls! { A B C D E F G H I J K L M N O P Q }
tuple_impls! { A B C D E F G H I J K L M N O P Q R }
tuple_impls! { A B C D E F G H I J K L M N O P Q R S }
tuple_impls! { A B C D E F G H I J K L M N O P Q R S T }
tuple_impls! { A B C D E F G H I J K L M N O P Q R S T U }
tuple_impls! { A B C D E F G H I J K L M N O P Q R S T U V }
tuple_impls! { A B C D E F G H I J K L M N O P Q R S T U V W }
tuple_impls! { A B C D E F G H I J K L M N O P Q R S T U V W X }
tuple_impls! { A B C D E F G H I J K L M N O P Q R S T U V W X Y }
tuple_impls! { A B C D E F G H I J K L M N O P Q R S T U V W X Y Z }
tuple_impls! { A B C D E F G H I J K L M N O P Q R S T U V W X Y Z AA }
tuple_impls! { A B C D E F G H I J K L M N O P Q R S T U V W X Y Z AA AB }
tuple_impls! { A B C D E F G H I J K L M N O P Q R S T U V W X Y Z AA AB AC }
tuple_impls! { A B C D E F G H I J K L M N O P Q R S T U V W X Y Z AA AB AC AD }
tuple_impls! { A B C D E F G H I J K L M N O P Q R S T U V W X Y Z AA AB AC AD AE }
tuple_impls! { A B C D E F G H I J K L M N O P Q R S T U V W X Y Z AA AB AC AD AE AF }
impl<T: SupportedValue> ComponentsTuple for Component<T> {
    type Data = T;

    fn as_indices(&self) -> Vec<ComponentIndex> {
        vec![self.index()]
    }
    fn from_component_types(component_types: Vec<wit::component::Value>) -> Option<Self::Data> {
        assert_eq!(component_types.len(), 1);
        T::from_result(component_types[0].clone())
    }
}
impl ComponentsTuple for () {
    type Data = ();

    fn as_indices(&self) -> Vec<ComponentIndex> {
        vec![]
    }
    fn from_component_types(component_types: Vec<wit::component::Value>) -> Option<Self::Data> {
        assert!(component_types.is_empty());
        Some(())
    }
}

/// Implemented for component values that can be used as an enum
pub trait EnumComponent {
    /// Convert this value to a u32
    fn to_u32(&self) -> u32;
    /// Convert a u32 to this value
    fn from_u32(v: u32) -> Option<Self>
    where
        Self: Sized;
}
