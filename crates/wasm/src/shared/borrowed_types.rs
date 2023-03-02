use super::wit::{
    self,
    component::{
        ComponentListTypeParam, ComponentListTypeResult, ComponentOptionTypeParam,
        ComponentOptionTypeResult, ComponentTypeParam, ComponentTypeResult,
    },
};

// We need to convert WIT's owned representation of `ComponentType` to the borrowed representation.
// Unfortunately, it contains types that contain other borrowed types, so we need an intermediate step:
//  Vec<String> -> Vec<&str> -> &[&str]
// This defines types for that intermediate step.

macro_rules! generate_borrowing_types_all {
    ($(($value:ident, $type:ty, $borrowed_type:ty)),*) => {
        paste::paste! {
            pub(super) enum ComponentListTypeBorrow<'a> {
                $([<Type $value>](Vec<$borrowed_type>),)*
            }
            impl<'a> From<&'a ComponentListTypeResult> for ComponentListTypeBorrow<'a> {
                fn from(owned: &'a ComponentListTypeResult) -> Self {
                    match owned {
                        $(ComponentListTypeResult::[<Type $value>](v) => ComponentListTypeBorrow::[<Type $value>](v.borrow_if_required()),)*
                    }
                }
            }
            impl<'a> ComponentListTypeBorrow<'a> {
                fn as_wit(&'a self) -> ComponentListTypeParam<'a> {
                    match self {
                        $(Self::[<Type $value>](v) => ComponentListTypeParam::[<Type $value>](v.as_slice()),)*
                    }
                }
            }

            pub(super) enum ComponentOptionTypeBorrow<'a> {
                $([<Type $value>](Option<$borrowed_type>),)*
            }
            impl<'a> From<&'a ComponentOptionTypeResult> for ComponentOptionTypeBorrow<'a> {
                fn from(owned: &'a ComponentOptionTypeResult) -> Self {
                    match owned {
                        $(ComponentOptionTypeResult::[<Type $value>](v) => ComponentOptionTypeBorrow::[<Type $value>](v.borrow_if_required()),)*
                    }
                }
            }
            impl<'a> ComponentOptionTypeBorrow<'a> {
                fn as_wit(&'a self) -> ComponentOptionTypeParam<'a> {
                    match self {
                        $(Self::[<Type $value>](v) => ComponentOptionTypeParam::[<Type $value>](*v),)*
                    }
                }
            }

            pub(super) enum ComponentTypeBorrow<'a> {
                $([<Type $value>]($borrowed_type),)*
                TypeList(ComponentListTypeBorrow<'a>),
                TypeOption(ComponentOptionTypeBorrow<'a>),
            }
            impl<'a> From<&'a ComponentTypeResult> for ComponentTypeBorrow<'a> {
                fn from(owned: &'a ComponentTypeResult) -> Self {
                    match owned {
                        $(ComponentTypeResult::[<Type $value>](v) => ComponentTypeBorrow::[<Type $value>](v.borrow_if_required()),)*
                        ComponentTypeResult::TypeList(v) => ComponentTypeBorrow::TypeList(v.into()),
                        ComponentTypeResult::TypeOption(v) => ComponentTypeBorrow::TypeOption(v.into()),
                    }
                }
            }
            impl<'a> ComponentTypeBorrow<'a> {
                pub fn as_wit(&'a self) -> ComponentTypeParam<'a> {
                    match self {
                        $(Self::[<Type $value>](v) => ComponentTypeParam::[<Type $value>](*v),)*
                        Self::TypeList(v) => ComponentTypeParam::TypeList(v.as_wit()),
                        Self::TypeOption(v) => ComponentTypeParam::TypeOption(v.as_wit()),
                    }
                }
            }
        }
    };
}

macro_rules! generate_borrowing_types {
    (
        owned: [$(($owned_value:ident, $owned_type:ty)),*],
        borrowed: [$(($borrowed_value:ident, $borrowed_owned_type:ty, $borrowed_type:ty)),*]
    ) => {
        $(
            impl<'a> BorrowIfRequired for &'a $owned_type {
                type Output = $owned_type;
                fn borrow_if_required(self) -> Self::Output {
                    *self
                }
            }
        )*
        $(
            impl<'a> BorrowIfRequired for &'a $borrowed_owned_type {
                type Output = $borrowed_type;
                fn borrow_if_required(self) -> Self::Output {
                    &*self
                }
            }
        )*

        generate_borrowing_types_all!(
            $(($owned_value, $owned_type, $owned_type),)*
            $(($borrowed_value, $borrowed_owned_type, $borrowed_type)),*
        );
    };
}

generate_borrowing_types!(
    owned: [
        (Empty, ()),
        (Bool, bool),
        (EntityId, wit::types::EntityId),
        (F32, f32),
        (F64, f64),
        (Mat4, wit::types::Mat4),
        (I32, i32),
        (Quat, wit::types::Quat),
        (U32, u32),
        (U64, u64),
        (Vec2, wit::types::Vec2),
        (Vec3, wit::types::Vec3),
        (Vec4, wit::types::Vec4)
    ],
    borrowed: [
        (String, String, &'a str)
    ]
);

trait BorrowIfRequired {
    type Output;
    fn borrow_if_required(self) -> Self::Output;
}
impl<'a, T> BorrowIfRequired for &'a Vec<T>
where
    &'a T: BorrowIfRequired,
{
    type Output = Vec<<&'a T as BorrowIfRequired>::Output>;
    fn borrow_if_required(self) -> Self::Output {
        self.iter().map(|v| v.borrow_if_required()).collect()
    }
}
impl<'a, T> BorrowIfRequired for &'a Option<T>
where
    &'a T: BorrowIfRequired,
{
    type Output = Option<<&'a T as BorrowIfRequired>::Output>;
    fn borrow_if_required(self) -> Self::Output {
        self.as_ref().map(|v| v.borrow_if_required())
    }
}
