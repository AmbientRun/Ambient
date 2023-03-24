use crate::{
    global::{EntityId, Mat4, Quat, Vec2, Vec3, Vec4},
    internal::{
        component::Component,
        conversion::{FromBindgen, IntoBindgen},
        wit,
    },
};
use glam::{UVec2, UVec3, UVec4};

#[doc(hidden)]
pub fn get_component<T>(id: &str) -> Component<T> {
    Component::new(wit::component::get_index(id).unwrap())
}

#[doc(hidden)]
pub trait AsParam {
    fn as_param(&self) -> wit::component::ValueParam<'_>;
}

/// Implemented by all types you can use with [entity::get_component](crate::entity::get_component).
pub trait SupportedValueGet
where
    Self: Sized,
{
    #[doc(hidden)]
    fn from_result(result: wit::component::ValueResult) -> Option<Self>;
}

/// Implemented by all types you can use with [entity::set_component](crate::entity::set_component).
pub trait SupportedValueSet
where
    Self: Sized,
{
    #[doc(hidden)]
    type OwnedParam: AsParam;

    #[doc(hidden)]
    fn into_result(self) -> wit::component::ValueResult;

    #[doc(hidden)]
    fn into_owned_param(self) -> Self::OwnedParam;
}

macro_rules! define_component_types {
    ($(($type:ty, $value:ident)),*) => {
        $(
        impl SupportedValueGet for $type {
            fn from_result(result: wit::component::ValueResult) -> Option<Self> {
                match result {
                    wit::component::ValueResult::$value(v) => Some(v.from_bindgen()),
                    _ => None,
                }
            }
        }

        impl SupportedValueSet for $type {
            type OwnedParam = Self;

            fn into_result(self) -> wit::component::ValueResult {
                wit::component::ValueResult::$value(self.into_bindgen())
            }

            fn into_owned_param(self) -> Self::OwnedParam {
                self
            }
        }

        impl AsParam for $type {
            fn as_param<'a>(&'a self) -> wit::component::ValueParam<'a> {
                wit::component::ValueParam::$value((*self).into_bindgen())
            }
        }
        ) *
    };
}

define_component_types!(
    ((), TypeEmpty),
    (bool, TypeBool),
    (EntityId, TypeEntityId),
    (f32, TypeF32),
    (f64, TypeF64),
    (Mat4, TypeMat4),
    (i32, TypeI32),
    (Quat, TypeQuat),
    (u8, TypeU8),
    (u32, TypeU32),
    (u64, TypeU64),
    (Vec2, TypeVec2),
    (Vec3, TypeVec3),
    (Vec4, TypeVec4),
    (UVec2, TypeUvec2),
    (UVec3, TypeUvec3),
    (UVec4, TypeUvec4)
);

impl SupportedValueGet for String {
    fn from_result(result: wit::component::ValueResult) -> Option<Self> {
        match result {
            wit::component::ValueResult::TypeString(v) => Some(v),
            _ => None,
        }
    }
}
impl SupportedValueSet for String {
    type OwnedParam = Self;

    fn into_result(self) -> wit::component::ValueResult {
        wit::component::ValueResult::TypeString(self.into_bindgen())
    }

    fn into_owned_param(self) -> Self::OwnedParam {
        self
    }
}
impl AsParam for String {
    fn as_param(&self) -> wit::component::ValueParam<'_> {
        wit::component::ValueParam::TypeString(self.as_str())
    }
}

macro_rules! define_vec_opt_component_types {
    ($(($type:ty, $value:ident)),*) => {
        $(
        impl SupportedValueGet for Vec<$type> {
            fn from_result(result: wit::component::ValueResult) -> Option<Self> {
                match result {
                    wit::component::ValueResult::TypeVec(wit::component::VecValueResult::$value(v)) => Some(v.into_iter().map(|v| v.from_bindgen()).collect()),
                    _ => None,
                }
            }
        }
        impl SupportedValueSet for Vec<$type> {
            type OwnedParam = Vec<<$type as IntoBindgen>::Item>;

            fn into_result(self) -> wit::component::ValueResult {
                wit::component::ValueResult::TypeVec(wit::component::VecValueResult::$value(self.into_bindgen()))
            }

            fn into_owned_param(self) -> Self::OwnedParam {
                self.iter().map(|v| (*v).into_bindgen()).collect()
            }
        }
        impl AsParam for Vec<<$type as IntoBindgen>::Item> {
            fn as_param(&self) -> wit::component::ValueParam<'_> {
                wit::component::ValueParam::TypeVec(wit::component::VecValueParam::$value(&self))
            }
        }

        impl SupportedValueGet for Option<$type> {
            fn from_result(result: wit::component::ValueResult) -> Option<Self> {
                match result {
                    wit::component::ValueResult::TypeOption(wit::component::OptionValueResult::$value(v)) => Some(v.from_bindgen()),
                    _ => None,
                }
            }
        }
        impl SupportedValueSet for Option<$type> {
            type OwnedParam = Option<<$type as IntoBindgen>::Item>;

            fn into_result(self) -> wit::component::ValueResult {
                wit::component::ValueResult::TypeOption(wit::component::OptionValueResult::$value(self.into_bindgen()))
            }

            fn into_owned_param(self) -> Self::OwnedParam {
                self.into_bindgen()
            }
        }
        impl AsParam for Option<<$type as IntoBindgen>::Item> {
            fn as_param(&self) -> wit::component::ValueParam<'_> {
                wit::component::ValueParam::TypeOption(wit::component::OptionValueParam::$value(self.clone()))
            }
        }
        ) *
    };
}

define_vec_opt_component_types!(
    ((), TypeEmpty),
    (bool, TypeBool),
    (EntityId, TypeEntityId),
    (f32, TypeF32),
    (f64, TypeF64),
    (Mat4, TypeMat4),
    (i32, TypeI32),
    (Quat, TypeQuat),
    (u8, TypeU8),
    (u32, TypeU32),
    (u64, TypeU64),
    (Vec2, TypeVec2),
    (Vec3, TypeVec3),
    (Vec4, TypeVec4),
    (UVec2, TypeUvec2),
    (UVec3, TypeUvec3),
    (UVec4, TypeUvec4)
);

impl SupportedValueGet for Vec<String> {
    fn from_result(result: wit::component::ValueResult) -> Option<Self> {
        match result {
            wit::component::ValueResult::TypeVec(wit::component::VecValueResult::TypeString(v)) => {
                Some(v.into_iter().map(|v| v.from_bindgen()).collect())
            }
            _ => None,
        }
    }
}
impl<'a> SupportedValueSet for &'a Vec<String> {
    type OwnedParam = Vec<&'a str>;

    fn into_result(self) -> wit::component::ValueResult {
        wit::component::ValueResult::TypeVec(wit::component::VecValueResult::TypeString(
            self.iter().map(|s| s.clone().into_bindgen()).collect(),
        ))
    }

    fn into_owned_param(self) -> Self::OwnedParam {
        self.iter().map(|v| v.as_str()).collect()
    }
}
impl<'a> AsParam for Vec<&'a str> {
    fn as_param(&self) -> wit::component::ValueParam<'_> {
        wit::component::ValueParam::TypeVec(wit::component::VecValueParam::TypeString(self))
    }
}

impl SupportedValueGet for Option<String> {
    fn from_result(result: wit::component::ValueResult) -> Option<Self> {
        match result {
            wit::component::ValueResult::TypeOption(
                wit::component::OptionValueResult::TypeString(v),
            ) => Some(v.from_bindgen()),
            _ => None,
        }
    }
}
impl<'a> SupportedValueSet for &'a Option<String> {
    type OwnedParam = Option<&'a str>;

    fn into_result(self) -> wit::component::ValueResult {
        wit::component::ValueResult::TypeOption(wit::component::OptionValueResult::TypeString(
            self.clone().into_bindgen(),
        ))
    }

    fn into_owned_param(self) -> Self::OwnedParam {
        self.as_ref().map(|s| s.as_str())
    }
}
impl<'a> AsParam for Option<&'a str> {
    fn as_param(&self) -> wit::component::ValueParam<'_> {
        wit::component::ValueParam::TypeOption(wit::component::OptionValueParam::TypeString(*self))
    }
}
