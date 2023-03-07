use crate::internal::wit::{
    self,
    component::{
        OptionValueParam, OptionValueResult, ValueParam, ValueResult, VecValueParam, VecValueResult,
    },
};

// We need to convert WIT's owned representation of `Value` to the borrowed representation.
// Unfortunately, it contains types that contain other borrowed types, so we need an intermediate step:
//  Vec<String> -> Vec<&str> -> &[&str]
// This defines types for that intermediate step.

macro_rules! generate_borrowing_types_all {
    ($(($value:ident, $type:ty, $borrowed_type:ty)),*) => {
        paste::paste! {
            pub(super) enum VecValueBorrow<'a> {
                $([<Type $value>](Vec<$borrowed_type>),)*
            }
            impl<'a> From<&'a VecValueResult> for VecValueBorrow<'a> {
                fn from(owned: &'a VecValueResult) -> Self {
                    match owned {
                        $(VecValueResult::[<Type $value>](v) => VecValueBorrow::[<Type $value>](v.borrow_if_required()),)*
                    }
                }
            }
            impl<'a> VecValueBorrow<'a> {
                fn as_wit(&'a self) -> VecValueParam<'a> {
                    match self {
                        $(Self::[<Type $value>](v) => VecValueParam::[<Type $value>](v.as_slice()),)*
                    }
                }
            }

            pub(super) enum OptionValueBorrow<'a> {
                $([<Type $value>](Option<$borrowed_type>),)*
            }
            impl<'a> From<&'a OptionValueResult> for OptionValueBorrow<'a> {
                fn from(owned: &'a OptionValueResult) -> Self {
                    match owned {
                        $(OptionValueResult::[<Type $value>](v) => OptionValueBorrow::[<Type $value>](v.borrow_if_required()),)*
                    }
                }
            }
            impl<'a> OptionValueBorrow<'a> {
                fn as_wit(&'a self) -> OptionValueParam<'a> {
                    match self {
                        $(Self::[<Type $value>](v) => OptionValueParam::[<Type $value>](*v),)*
                    }
                }
            }

            pub(super) enum ValueBorrow<'a> {
                $([<Type $value>]($borrowed_type),)*
                TypeVec(VecValueBorrow<'a>),
                TypeOption(OptionValueBorrow<'a>),
            }
            impl<'a> From<&'a ValueResult> for ValueBorrow<'a> {
                fn from(owned: &'a ValueResult) -> Self {
                    match owned {
                        $(ValueResult::[<Type $value>](v) => ValueBorrow::[<Type $value>](v.borrow_if_required()),)*
                        ValueResult::TypeVec(v) => ValueBorrow::TypeVec(v.into()),
                        ValueResult::TypeOption(v) => ValueBorrow::TypeOption(v.into()),
                    }
                }
            }
            impl<'a> ValueBorrow<'a> {
                pub fn as_wit(&'a self) -> ValueParam<'a> {
                    match self {
                        $(Self::[<Type $value>](v) => ValueParam::[<Type $value>](*v),)*
                        Self::TypeVec(v) => ValueParam::TypeVec(v.as_wit()),
                        Self::TypeOption(v) => ValueParam::TypeOption(v.as_wit()),
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
        (Vec4, wit::types::Vec4),
        (Uvec2, wit::types::Uvec2),
        (Uvec3, wit::types::Uvec3),
        (Uvec4, wit::types::Uvec4)
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
