use crate::{
    global::{EntityId, Mat4, Quat, Vec2, Vec3, Vec4},
    internal::{
        component::Component,
        conversion::{FromBindgen, IntoBindgen},
        wit,
    },
};

#[doc(hidden)]
pub fn get_component<T>(id: &str) -> Component<T> {
    Component::new(wit::component::get_index(id).unwrap())
}

#[doc(hidden)]
pub trait AsParam {
    fn as_param(&self) -> wit::component::ComponentTypeParam<'_>;
}

/// Implemented by all types you can use with [entity::get_component](crate::entity::get_component).
pub trait SupportedComponentTypeGet
where
    Self: Sized,
{
    #[doc(hidden)]
    fn from_result(result: wit::component::ComponentTypeResult) -> Option<Self>;
}

/// Implemented by all types you can use with [entity::set_component](crate::entity::set_component).
pub trait SupportedComponentTypeSet
where
    Self: Sized,
{
    #[doc(hidden)]
    type OwnedParam: AsParam;

    #[doc(hidden)]
    fn into_result(self) -> wit::component::ComponentTypeResult;

    #[doc(hidden)]
    fn into_owned_param(self) -> Self::OwnedParam;
}

macro_rules! define_component_types {
    ($(($type:ty, $value:ident)),*) => {
        $(
        impl SupportedComponentTypeGet for $type {
            fn from_result(result: wit::component::ComponentTypeResult) -> Option<Self> {
                match result {
                    wit::component::ComponentTypeResult::$value(v) => Some(v.from_bindgen()),
                    _ => None,
                }
            }
        }

        impl SupportedComponentTypeSet for $type {
            type OwnedParam = Self;

            fn into_result(self) -> wit::component::ComponentTypeResult {
                wit::component::ComponentTypeResult::$value(self.into_bindgen())
            }

            fn into_owned_param(self) -> Self::OwnedParam {
                self
            }
        }

        impl AsParam for $type {
            fn as_param<'a>(&'a self) -> wit::component::ComponentTypeParam<'a> {
                wit::component::ComponentTypeParam::$value((*self).into_bindgen())
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
    (u32, TypeU32),
    (u64, TypeU64),
    (Vec2, TypeVec2),
    (Vec3, TypeVec3),
    (Vec4, TypeVec4)
);

impl SupportedComponentTypeGet for String {
    fn from_result(result: wit::component::ComponentTypeResult) -> Option<Self> {
        match result {
            wit::component::ComponentTypeResult::TypeString(v) => Some(v),
            _ => None,
        }
    }
}
impl SupportedComponentTypeSet for String {
    type OwnedParam = Self;

    fn into_result(self) -> wit::component::ComponentTypeResult {
        wit::component::ComponentTypeResult::TypeString(self.into_bindgen())
    }

    fn into_owned_param(self) -> Self::OwnedParam {
        self
    }
}
impl AsParam for String {
    fn as_param(&self) -> wit::component::ComponentTypeParam<'_> {
        wit::component::ComponentTypeParam::TypeString(self.as_str())
    }
}

macro_rules! define_vec_opt_component_types {
    ($(($type:ty, $value:ident)),*) => {
        $(
        impl SupportedComponentTypeGet for Vec<$type> {
            fn from_result(result: wit::component::ComponentTypeResult) -> Option<Self> {
                match result {
                    wit::component::ComponentTypeResult::TypeList(wit::component::ComponentListTypeResult::$value(v)) => Some(v.into_iter().map(|v| v.from_bindgen()).collect()),
                    _ => None,
                }
            }
        }
        impl SupportedComponentTypeSet for Vec<$type> {
            type OwnedParam = Vec<<$type as IntoBindgen>::Item>;

            fn into_result(self) -> wit::component::ComponentTypeResult {
                wit::component::ComponentTypeResult::TypeList(wit::component::ComponentListTypeResult::$value(self.into_bindgen()))
            }

            fn into_owned_param(self) -> Self::OwnedParam {
                self.iter().map(|v| (*v).into_bindgen()).collect()
            }
        }
        impl AsParam for Vec<<$type as IntoBindgen>::Item> {
            fn as_param(&self) -> wit::component::ComponentTypeParam<'_> {
                wit::component::ComponentTypeParam::TypeList(wit::component::ComponentListTypeParam::$value(&self))
            }
        }

        impl SupportedComponentTypeGet for Option<$type> {
            fn from_result(result: wit::component::ComponentTypeResult) -> Option<Self> {
                match result {
                    wit::component::ComponentTypeResult::TypeOption(wit::component::ComponentOptionTypeResult::$value(v)) => Some(v.from_bindgen()),
                    _ => None,
                }
            }
        }
        impl SupportedComponentTypeSet for Option<$type> {
            type OwnedParam = Option<<$type as IntoBindgen>::Item>;

            fn into_result(self) -> wit::component::ComponentTypeResult {
                wit::component::ComponentTypeResult::TypeOption(wit::component::ComponentOptionTypeResult::$value(self.into_bindgen()))
            }

            fn into_owned_param(self) -> Self::OwnedParam {
                self.into_bindgen()
            }
        }
        impl AsParam for Option<<$type as IntoBindgen>::Item> {
            fn as_param(&self) -> wit::component::ComponentTypeParam<'_> {
                wit::component::ComponentTypeParam::TypeOption(wit::component::ComponentOptionTypeParam::$value(self.clone()))
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
    (u32, TypeU32),
    (u64, TypeU64),
    (Vec2, TypeVec2),
    (Vec3, TypeVec3),
    (Vec4, TypeVec4)
);

impl SupportedComponentTypeGet for Vec<String> {
    fn from_result(result: wit::component::ComponentTypeResult) -> Option<Self> {
        match result {
            wit::component::ComponentTypeResult::TypeList(
                wit::component::ComponentListTypeResult::TypeString(v),
            ) => Some(v.into_iter().map(|v| v.from_bindgen()).collect()),
            _ => None,
        }
    }
}
impl<'a> SupportedComponentTypeSet for &'a Vec<String> {
    type OwnedParam = Vec<&'a str>;

    fn into_result(self) -> wit::component::ComponentTypeResult {
        wit::component::ComponentTypeResult::TypeList(
            wit::component::ComponentListTypeResult::TypeString(
                self.iter().map(|s| s.clone().into_bindgen()).collect(),
            ),
        )
    }

    fn into_owned_param(self) -> Self::OwnedParam {
        self.iter().map(|v| v.as_str()).collect()
    }
}
impl<'a> AsParam for Vec<&'a str> {
    fn as_param(&self) -> wit::component::ComponentTypeParam<'_> {
        wit::component::ComponentTypeParam::TypeList(
            wit::component::ComponentListTypeParam::TypeString(self),
        )
    }
}

impl SupportedComponentTypeGet for Option<String> {
    fn from_result(result: wit::component::ComponentTypeResult) -> Option<Self> {
        match result {
            wit::component::ComponentTypeResult::TypeOption(
                wit::component::ComponentOptionTypeResult::TypeString(v),
            ) => Some(v.from_bindgen()),
            _ => None,
        }
    }
}
impl<'a> SupportedComponentTypeSet for &'a Option<String> {
    type OwnedParam = Option<&'a str>;

    fn into_result(self) -> wit::component::ComponentTypeResult {
        wit::component::ComponentTypeResult::TypeOption(
            wit::component::ComponentOptionTypeResult::TypeString(self.clone().into_bindgen()),
        )
    }

    fn into_owned_param(self) -> Self::OwnedParam {
        self.as_ref().map(|s| s.as_str())
    }
}
impl<'a> AsParam for Option<&'a str> {
    fn as_param(&self) -> wit::component::ComponentTypeParam<'_> {
        wit::component::ComponentTypeParam::TypeOption(
            wit::component::ComponentOptionTypeParam::TypeString(*self),
        )
    }
}
