use crate::{
    global::{EntityId, EntityUid, Mat4, ObjectRef, Quat, Vec2, Vec3, Vec4},
    internal::{
        component::Component,
        conversion::{FromBindgen, IntoBindgen},
        host,
    },
};

#[doc(hidden)]
pub fn get_component<T>(id: &str) -> Component<T> {
    Component::new(host::component_get_index(id).unwrap())
}

#[doc(hidden)]
pub trait AsParam {
    fn as_param(&self) -> host::ComponentTypeParam<'_>;
}

/// Implemented by all types you can use with [entity::get_component](crate::entity::get_component).
pub trait SupportedComponentTypeGet
where
    Self: Sized,
{
    #[doc(hidden)]
    fn from_result(result: host::ComponentTypeResult) -> Option<Self>;
}

/// Implemented by all types you can use with [entity::set_component](crate::entity::set_component).
pub trait SupportedComponentTypeSet
where
    Self: Sized,
{
    #[doc(hidden)]
    type OwnedParam: AsParam;

    #[doc(hidden)]
    fn into_result(self) -> host::ComponentTypeResult;

    #[doc(hidden)]
    fn into_owned_param(self) -> Self::OwnedParam;
}

macro_rules! define_component_types {
    ($(($type:ty, $value:ident)),*) => {
        $(
        impl SupportedComponentTypeGet for $type {
            fn from_result(result: host::ComponentTypeResult) -> Option<Self> {
                match result {
                    host::ComponentTypeResult::$value(v) => Some(v.from_bindgen()),
                    _ => None,
                }
            }
        }

        impl SupportedComponentTypeSet for $type {
            type OwnedParam = Self;

            fn into_result(self) -> host::ComponentTypeResult {
                host::ComponentTypeResult::$value(self.into_bindgen())
            }

            fn into_owned_param(self) -> Self::OwnedParam {
                self
            }
        }

        impl AsParam for $type {
            fn as_param<'a>(&'a self) -> host::ComponentTypeParam<'a> {
                host::ComponentTypeParam::$value((*self).into_bindgen())
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

impl SupportedComponentTypeGet for ObjectRef {
    fn from_result(result: host::ComponentTypeResult) -> Option<Self> {
        match result {
            host::ComponentTypeResult::TypeObjectRef(v) => Some(v.from_bindgen()),
            _ => None,
        }
    }
}
impl SupportedComponentTypeSet for ObjectRef {
    type OwnedParam = Self;

    fn into_result(self) -> host::ComponentTypeResult {
        host::ComponentTypeResult::TypeObjectRef(self.into_bindgen())
    }

    fn into_owned_param(self) -> Self::OwnedParam {
        self
    }
}
impl AsParam for ObjectRef {
    fn as_param(&self) -> host::ComponentTypeParam<'_> {
        host::ComponentTypeParam::TypeObjectRef(self.into_bindgen())
    }
}

impl SupportedComponentTypeGet for EntityUid {
    fn from_result(result: host::ComponentTypeResult) -> Option<Self> {
        match result {
            host::ComponentTypeResult::TypeEntityUid(v) => Some(v.from_bindgen()),
            _ => None,
        }
    }
}
impl SupportedComponentTypeSet for EntityUid {
    type OwnedParam = Self;

    fn into_result(self) -> host::ComponentTypeResult {
        host::ComponentTypeResult::TypeEntityUid(self.into_bindgen())
    }

    fn into_owned_param(self) -> Self::OwnedParam {
        self
    }
}
impl AsParam for EntityUid {
    fn as_param(&self) -> host::ComponentTypeParam<'_> {
        host::ComponentTypeParam::TypeEntityUid(self.into_bindgen())
    }
}

impl SupportedComponentTypeGet for String {
    fn from_result(result: host::ComponentTypeResult) -> Option<Self> {
        match result {
            host::ComponentTypeResult::TypeString(v) => Some(v),
            _ => None,
        }
    }
}
impl SupportedComponentTypeSet for String {
    type OwnedParam = Self;

    fn into_result(self) -> host::ComponentTypeResult {
        host::ComponentTypeResult::TypeString(self.into_bindgen())
    }

    fn into_owned_param(self) -> Self::OwnedParam {
        self
    }
}
impl AsParam for String {
    fn as_param(&self) -> host::ComponentTypeParam<'_> {
        host::ComponentTypeParam::TypeString(self.as_str())
    }
}

macro_rules! define_vec_opt_component_types {
    ($(($type:ty, $value:ident)),*) => {
        $(
        impl SupportedComponentTypeGet for Vec<$type> {
            fn from_result(result: host::ComponentTypeResult) -> Option<Self> {
                match result {
                    host::ComponentTypeResult::TypeList(host::ComponentListTypeResult::$value(v)) => Some(v.into_iter().map(|v| v.from_bindgen()).collect()),
                    _ => None,
                }
            }
        }
        impl SupportedComponentTypeSet for Vec<$type> {
            type OwnedParam = Vec<<$type as IntoBindgen>::Item>;

            fn into_result(self) -> host::ComponentTypeResult {
                host::ComponentTypeResult::TypeList(host::ComponentListTypeResult::$value(self.into_bindgen()))
            }

            fn into_owned_param(self) -> Self::OwnedParam {
                self.iter().map(|v| (*v).into_bindgen()).collect()
            }
        }
        impl AsParam for Vec<<$type as IntoBindgen>::Item> {
            fn as_param(&self) -> host::ComponentTypeParam<'_> {
                host::ComponentTypeParam::TypeList(host::ComponentListTypeParam::$value(&self))
            }
        }

        impl SupportedComponentTypeGet for Option<$type> {
            fn from_result(result: host::ComponentTypeResult) -> Option<Self> {
                match result {
                    host::ComponentTypeResult::TypeOption(host::ComponentOptionTypeResult::$value(v)) => Some(v.from_bindgen()),
                    _ => None,
                }
            }
        }
        impl SupportedComponentTypeSet for Option<$type> {
            type OwnedParam = Option<<$type as IntoBindgen>::Item>;

            fn into_result(self) -> host::ComponentTypeResult {
                host::ComponentTypeResult::TypeOption(host::ComponentOptionTypeResult::$value(self.into_bindgen()))
            }

            fn into_owned_param(self) -> Self::OwnedParam {
                self.into_bindgen()
            }
        }
        impl AsParam for Option<<$type as IntoBindgen>::Item> {
            fn as_param(&self) -> host::ComponentTypeParam<'_> {
                host::ComponentTypeParam::TypeOption(host::ComponentOptionTypeParam::$value(self.clone()))
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

impl SupportedComponentTypeGet for Vec<ObjectRef> {
    fn from_result(result: host::ComponentTypeResult) -> Option<Self> {
        match result {
            host::ComponentTypeResult::TypeList(host::ComponentListTypeResult::TypeObjectRef(
                v,
            )) => Some(v.into_iter().map(|v| v.from_bindgen()).collect()),
            _ => None,
        }
    }
}
impl<'a> SupportedComponentTypeSet for &'a Vec<ObjectRef> {
    type OwnedParam = Vec<host::ObjectRefParam<'a>>;

    fn into_result(self) -> host::ComponentTypeResult {
        host::ComponentTypeResult::TypeList(host::ComponentListTypeResult::TypeObjectRef(
            self.iter().map(|s| s.clone().into_bindgen()).collect(),
        ))
    }

    fn into_owned_param(self) -> Self::OwnedParam {
        self.iter()
            .map(|v| host::ObjectRefParam { id: v.as_ref() })
            .collect()
    }
}
impl<'a> AsParam for Vec<host::ObjectRefParam<'a>> {
    fn as_param(&self) -> host::ComponentTypeParam<'_> {
        host::ComponentTypeParam::TypeList(host::ComponentListTypeParam::TypeObjectRef(self))
    }
}

impl SupportedComponentTypeGet for Option<ObjectRef> {
    fn from_result(result: host::ComponentTypeResult) -> Option<Self> {
        match result {
            host::ComponentTypeResult::TypeOption(
                host::ComponentOptionTypeResult::TypeObjectRef(v),
            ) => Some(v.from_bindgen()),
            _ => None,
        }
    }
}
impl<'a> SupportedComponentTypeSet for &'a Option<ObjectRef> {
    type OwnedParam = Option<host::ObjectRefParam<'a>>;

    fn into_result(self) -> host::ComponentTypeResult {
        host::ComponentTypeResult::TypeOption(host::ComponentOptionTypeResult::TypeObjectRef(
            self.clone().into_bindgen(),
        ))
    }

    fn into_owned_param(self) -> Self::OwnedParam {
        self.as_ref()
            .map(|s| host::ObjectRefParam { id: s.as_ref() })
    }
}
impl<'a> AsParam for Option<host::ObjectRefParam<'a>> {
    fn as_param(&self) -> host::ComponentTypeParam<'_> {
        host::ComponentTypeParam::TypeOption(host::ComponentOptionTypeParam::TypeObjectRef(
            self.clone(),
        ))
    }
}

impl SupportedComponentTypeGet for Vec<EntityUid> {
    fn from_result(result: host::ComponentTypeResult) -> Option<Self> {
        match result {
            host::ComponentTypeResult::TypeList(host::ComponentListTypeResult::TypeEntityUid(
                v,
            )) => Some(v.into_iter().map(|v| v.from_bindgen()).collect()),
            _ => None,
        }
    }
}
impl<'a> SupportedComponentTypeSet for &'a Vec<EntityUid> {
    type OwnedParam = Vec<host::EntityUidParam<'a>>;

    fn into_result(self) -> host::ComponentTypeResult {
        host::ComponentTypeResult::TypeList(host::ComponentListTypeResult::TypeEntityUid(
            self.iter().map(|s| s.clone().into_bindgen()).collect(),
        ))
    }

    fn into_owned_param(self) -> Self::OwnedParam {
        self.iter()
            .map(|v| host::EntityUidParam { id: v.as_ref() })
            .collect()
    }
}
impl<'a> AsParam for Vec<host::EntityUidParam<'a>> {
    fn as_param(&self) -> host::ComponentTypeParam<'_> {
        host::ComponentTypeParam::TypeList(host::ComponentListTypeParam::TypeEntityUid(self))
    }
}

impl SupportedComponentTypeGet for Option<EntityUid> {
    fn from_result(result: host::ComponentTypeResult) -> Option<Self> {
        match result {
            host::ComponentTypeResult::TypeOption(
                host::ComponentOptionTypeResult::TypeEntityUid(v),
            ) => Some(v.from_bindgen()),
            _ => None,
        }
    }
}
impl<'a> SupportedComponentTypeSet for &'a Option<EntityUid> {
    type OwnedParam = Option<host::EntityUidParam<'a>>;

    fn into_result(self) -> host::ComponentTypeResult {
        host::ComponentTypeResult::TypeOption(host::ComponentOptionTypeResult::TypeEntityUid(
            self.clone().into_bindgen(),
        ))
    }

    fn into_owned_param(self) -> Self::OwnedParam {
        self.as_ref()
            .map(|s| host::EntityUidParam { id: s.as_ref() })
    }
}
impl<'a> AsParam for Option<host::EntityUidParam<'a>> {
    fn as_param(&self) -> host::ComponentTypeParam<'_> {
        host::ComponentTypeParam::TypeOption(host::ComponentOptionTypeParam::TypeEntityUid(
            self.clone(),
        ))
    }
}

impl SupportedComponentTypeGet for Vec<String> {
    fn from_result(result: host::ComponentTypeResult) -> Option<Self> {
        match result {
            host::ComponentTypeResult::TypeList(host::ComponentListTypeResult::TypeString(v)) => {
                Some(v.into_iter().map(|v| v.from_bindgen()).collect())
            }
            _ => None,
        }
    }
}
impl<'a> SupportedComponentTypeSet for &'a Vec<String> {
    type OwnedParam = Vec<&'a str>;

    fn into_result(self) -> host::ComponentTypeResult {
        host::ComponentTypeResult::TypeList(host::ComponentListTypeResult::TypeString(
            self.iter().map(|s| s.clone().into_bindgen()).collect(),
        ))
    }

    fn into_owned_param(self) -> Self::OwnedParam {
        self.iter().map(|v| v.as_str()).collect()
    }
}
impl<'a> AsParam for Vec<&'a str> {
    fn as_param(&self) -> host::ComponentTypeParam<'_> {
        host::ComponentTypeParam::TypeList(host::ComponentListTypeParam::TypeString(self))
    }
}

impl SupportedComponentTypeGet for Option<String> {
    fn from_result(result: host::ComponentTypeResult) -> Option<Self> {
        match result {
            host::ComponentTypeResult::TypeOption(host::ComponentOptionTypeResult::TypeString(
                v,
            )) => Some(v.from_bindgen()),
            _ => None,
        }
    }
}
impl<'a> SupportedComponentTypeSet for &'a Option<String> {
    type OwnedParam = Option<&'a str>;

    fn into_result(self) -> host::ComponentTypeResult {
        host::ComponentTypeResult::TypeOption(host::ComponentOptionTypeResult::TypeString(
            self.clone().into_bindgen(),
        ))
    }

    fn into_owned_param(self) -> Self::OwnedParam {
        self.as_ref().map(|s| s.as_str())
    }
}
impl<'a> AsParam for Option<&'a str> {
    fn as_param(&self) -> host::ComponentTypeParam<'_> {
        host::ComponentTypeParam::TypeOption(host::ComponentOptionTypeParam::TypeString(*self))
    }
}
