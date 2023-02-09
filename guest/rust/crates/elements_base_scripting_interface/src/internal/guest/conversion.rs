//! temporary as the version of wit-bindgen we're using does not unify types betwen imports
//! and exports
//! once we use the component model, this can be deleted

use super::guest;
use crate::host;

pub(crate) trait GuestConvert {
    type Item;
    fn guest_convert(self) -> Self::Item;
}

impl GuestConvert for guest::EntityId {
    type Item = host::EntityId;
    fn guest_convert(self) -> Self::Item {
        Self::Item {
            id0: self.id0,
            id1: self.id1,
        }
    }
}

impl GuestConvert for guest::Vec2 {
    type Item = host::Vec2;
    fn guest_convert(self) -> Self::Item {
        Self::Item {
            x: self.x,
            y: self.y,
        }
    }
}

impl GuestConvert for guest::Vec3 {
    type Item = host::Vec3;
    fn guest_convert(self) -> Self::Item {
        Self::Item {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

impl GuestConvert for guest::Vec4 {
    type Item = host::Vec4;
    fn guest_convert(self) -> Self::Item {
        Self::Item {
            x: self.x,
            y: self.y,
            z: self.z,
            w: self.w,
        }
    }
}

impl GuestConvert for guest::Quat {
    type Item = host::Quat;
    fn guest_convert(self) -> Self::Item {
        Self::Item {
            x: self.x,
            y: self.y,
            z: self.z,
            w: self.w,
        }
    }
}

impl GuestConvert for guest::Mat4 {
    type Item = host::Mat4;
    fn guest_convert(self) -> Self::Item {
        Self::Item {
            x: self.x.guest_convert(),
            y: self.y.guest_convert(),
            z: self.z.guest_convert(),
            w: self.w.guest_convert(),
        }
    }
}

impl GuestConvert for guest::ObjectRef {
    type Item = host::ObjectRefResult;
    fn guest_convert(self) -> Self::Item {
        Self::Item { id: self.id }
    }
}

impl GuestConvert for guest::EntityUid {
    type Item = host::EntityUidResult;
    fn guest_convert(self) -> Self::Item {
        Self::Item { id: self.id }
    }
}

macro_rules! convert_passthrough {
    ($type:ty) => {
        impl GuestConvert for $type {
            type Item = Self;
            fn guest_convert(self) -> Self::Item {
                self
            }
        }
    };
}

convert_passthrough!(());
convert_passthrough!(bool);
convert_passthrough!(f32);
convert_passthrough!(f64);
convert_passthrough!(i32);
convert_passthrough!(String);
convert_passthrough!(u32);
convert_passthrough!(u64);

impl<T> GuestConvert for Option<T>
where
    T: GuestConvert,
{
    type Item = Option<T::Item>;
    fn guest_convert(self) -> Self::Item {
        self.map(|i| i.guest_convert())
    }
}

impl<T> GuestConvert for Vec<T>
where
    T: GuestConvert,
{
    type Item = Vec<T::Item>;
    fn guest_convert(self) -> Self::Item {
        self.into_iter().map(|i| i.guest_convert()).collect()
    }
}

impl GuestConvert for guest::ComponentListType {
    type Item = host::ComponentListTypeResult;
    fn guest_convert(self) -> Self::Item {
        match self {
            Self::TypeEmpty(c) => Self::Item::TypeEmpty(c.guest_convert()),
            Self::TypeBool(c) => Self::Item::TypeBool(c.guest_convert()),
            Self::TypeEntityId(c) => Self::Item::TypeEntityId(c.guest_convert()),
            Self::TypeF32(c) => Self::Item::TypeF32(c.guest_convert()),
            Self::TypeF64(c) => Self::Item::TypeF64(c.guest_convert()),
            Self::TypeMat4(c) => Self::Item::TypeMat4(c.guest_convert()),
            Self::TypeI32(c) => Self::Item::TypeI32(c.guest_convert()),
            Self::TypeQuat(c) => Self::Item::TypeQuat(c.guest_convert()),
            Self::TypeString(c) => Self::Item::TypeString(c.guest_convert()),
            Self::TypeU32(c) => Self::Item::TypeU32(c.guest_convert()),
            Self::TypeU64(c) => Self::Item::TypeU64(c.guest_convert()),
            Self::TypeVec2(c) => Self::Item::TypeVec2(c.guest_convert()),
            Self::TypeVec3(c) => Self::Item::TypeVec3(c.guest_convert()),
            Self::TypeVec4(c) => Self::Item::TypeVec4(c.guest_convert()),
            Self::TypeObjectRef(c) => Self::Item::TypeObjectRef(c.guest_convert()),
            Self::TypeEntityUid(c) => Self::Item::TypeEntityUid(c.guest_convert()),
        }
    }
}

impl GuestConvert for guest::ComponentOptionType {
    type Item = host::ComponentOptionTypeResult;
    fn guest_convert(self) -> Self::Item {
        match self {
            Self::TypeEmpty(c) => Self::Item::TypeEmpty(c.guest_convert()),
            Self::TypeBool(c) => Self::Item::TypeBool(c.guest_convert()),
            Self::TypeEntityId(c) => Self::Item::TypeEntityId(c.guest_convert()),
            Self::TypeF32(c) => Self::Item::TypeF32(c.guest_convert()),
            Self::TypeF64(c) => Self::Item::TypeF64(c.guest_convert()),
            Self::TypeMat4(c) => Self::Item::TypeMat4(c.guest_convert()),
            Self::TypeI32(c) => Self::Item::TypeI32(c.guest_convert()),
            Self::TypeQuat(c) => Self::Item::TypeQuat(c.guest_convert()),
            Self::TypeString(c) => Self::Item::TypeString(c.guest_convert()),
            Self::TypeU32(c) => Self::Item::TypeU32(c.guest_convert()),
            Self::TypeU64(c) => Self::Item::TypeU64(c.guest_convert()),
            Self::TypeVec2(c) => Self::Item::TypeVec2(c.guest_convert()),
            Self::TypeVec3(c) => Self::Item::TypeVec3(c.guest_convert()),
            Self::TypeVec4(c) => Self::Item::TypeVec4(c.guest_convert()),
            Self::TypeObjectRef(c) => Self::Item::TypeObjectRef(c.guest_convert()),
            Self::TypeEntityUid(c) => Self::Item::TypeEntityUid(c.guest_convert()),
        }
    }
}

impl GuestConvert for guest::ComponentType {
    type Item = host::ComponentTypeResult;
    fn guest_convert(self) -> Self::Item {
        match self {
            Self::TypeEmpty(_) => Self::Item::TypeEmpty(()),
            Self::TypeBool(c) => Self::Item::TypeBool(c.guest_convert()),
            Self::TypeEntityId(c) => Self::Item::TypeEntityId(c.guest_convert()),
            Self::TypeF32(c) => Self::Item::TypeF32(c.guest_convert()),
            Self::TypeF64(c) => Self::Item::TypeF64(c.guest_convert()),
            Self::TypeMat4(c) => Self::Item::TypeMat4(c.guest_convert()),
            Self::TypeI32(c) => Self::Item::TypeI32(c.guest_convert()),
            Self::TypeQuat(c) => Self::Item::TypeQuat(c.guest_convert()),
            Self::TypeString(c) => Self::Item::TypeString(c.guest_convert()),
            Self::TypeU32(c) => Self::Item::TypeU32(c.guest_convert()),
            Self::TypeU64(c) => Self::Item::TypeU64(c.guest_convert()),
            Self::TypeVec2(c) => Self::Item::TypeVec2(c.guest_convert()),
            Self::TypeVec3(c) => Self::Item::TypeVec3(c.guest_convert()),
            Self::TypeVec4(c) => Self::Item::TypeVec4(c.guest_convert()),
            Self::TypeObjectRef(c) => Self::Item::TypeObjectRef(c.guest_convert()),
            Self::TypeEntityUid(c) => Self::Item::TypeEntityUid(c.guest_convert()),
            Self::TypeList(c) => Self::Item::TypeList(c.guest_convert()),
            Self::TypeOption(c) => Self::Item::TypeOption(c.guest_convert()),
        }
    }
}
