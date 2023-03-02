//! temporary as the version of wit-bindgen we're using does not unify types betwen imports
//! and exports
//! once we use the component model, this can be deleted

use super::interface::{guest, host};
pub(crate) trait GuestConvert {
    type Item;
    fn guest_convert(self) -> Self::Item;
}

impl GuestConvert for host::EntityId {
    type Item = guest::EntityId;
    fn guest_convert(self) -> Self::Item {
        Self::Item {
            id0: self.id0,
            id1: self.id1,
        }
    }
}

impl GuestConvert for host::Vec2 {
    type Item = guest::Vec2;
    fn guest_convert(self) -> Self::Item {
        Self::Item {
            x: self.x,
            y: self.y,
        }
    }
}

impl GuestConvert for host::Vec3 {
    type Item = guest::Vec3;
    fn guest_convert(self) -> Self::Item {
        Self::Item {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

impl GuestConvert for host::Vec4 {
    type Item = guest::Vec4;
    fn guest_convert(self) -> Self::Item {
        Self::Item {
            x: self.x,
            y: self.y,
            z: self.z,
            w: self.w,
        }
    }
}

impl GuestConvert for host::Uvec2 {
    type Item = guest::Uvec2;
    fn guest_convert(self) -> Self::Item {
        Self::Item {
            x: self.x,
            y: self.y,
        }
    }
}

impl GuestConvert for host::Uvec3 {
    type Item = guest::Uvec3;
    fn guest_convert(self) -> Self::Item {
        Self::Item {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

impl GuestConvert for host::Uvec4 {
    type Item = guest::Uvec4;
    fn guest_convert(self) -> Self::Item {
        Self::Item {
            x: self.x,
            y: self.y,
            z: self.z,
            w: self.w,
        }
    }
}

impl GuestConvert for host::Quat {
    type Item = guest::Quat;
    fn guest_convert(self) -> Self::Item {
        Self::Item {
            x: self.x,
            y: self.y,
            z: self.z,
            w: self.w,
        }
    }
}

impl GuestConvert for host::Mat4 {
    type Item = guest::Mat4;
    fn guest_convert(self) -> Self::Item {
        Self::Item {
            x: self.x.guest_convert(),
            y: self.y.guest_convert(),
            z: self.z.guest_convert(),
            w: self.w.guest_convert(),
        }
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

// aaaaaaarghhhhhhh
// wit-bindgen generates borrowing types so I need to define my owned types for the borrowing types to borrow from
// really looking forward to not having to do this!

pub(crate) enum ComponentListType<'a> {
    TypeEmpty(Vec<()>),
    TypeBool(Vec<bool>),
    TypeEntityId(Vec<guest::EntityId>),
    TypeF32(Vec<f32>),
    TypeF64(Vec<f64>),
    TypeMat4(Vec<guest::Mat4>),
    TypeI32(Vec<i32>),
    TypeQuat(Vec<guest::Quat>),
    TypeString(Vec<&'a str>),
    TypeU32(Vec<u32>),
    TypeU64(Vec<u64>),
    TypeVec2(Vec<guest::Vec2>),
    TypeVec3(Vec<guest::Vec3>),
    TypeVec4(Vec<guest::Vec4>),
    TypeUVec2(Vec<guest::Uvec2>),
    TypeUVec3(Vec<guest::Uvec3>),
    TypeUVec4(Vec<guest::Uvec4>),
}
impl<'a> ComponentListType<'a> {
    pub fn as_guest(&'a self) -> guest::ComponentListType<'a> {
        match self {
            Self::TypeEmpty(v) => guest::ComponentListType::TypeEmpty(v),
            Self::TypeBool(v) => guest::ComponentListType::TypeBool(v),
            Self::TypeEntityId(v) => guest::ComponentListType::TypeEntityId(v),
            Self::TypeF32(v) => guest::ComponentListType::TypeF32(v),
            Self::TypeF64(v) => guest::ComponentListType::TypeF64(v),
            Self::TypeMat4(v) => guest::ComponentListType::TypeMat4(v),
            Self::TypeI32(v) => guest::ComponentListType::TypeI32(v),
            Self::TypeQuat(v) => guest::ComponentListType::TypeQuat(v),
            Self::TypeString(v) => guest::ComponentListType::TypeString(v),
            Self::TypeU32(v) => guest::ComponentListType::TypeU32(v),
            Self::TypeU64(v) => guest::ComponentListType::TypeU64(v),
            Self::TypeVec2(v) => guest::ComponentListType::TypeVec2(v),
            Self::TypeVec3(v) => guest::ComponentListType::TypeVec3(v),
            Self::TypeVec4(v) => guest::ComponentListType::TypeVec4(v),
            Self::TypeUVec2(v) => guest::ComponentListType::TypeUvec2(v),
            Self::TypeUVec3(v) => guest::ComponentListType::TypeUvec3(v),
            Self::TypeUVec4(v) => guest::ComponentListType::TypeUvec4(v),
        }
    }
}

pub(crate) enum ComponentOptionType<'a> {
    TypeEmpty(Option<()>),
    TypeBool(Option<bool>),
    TypeEntityId(Option<guest::EntityId>),
    TypeF32(Option<f32>),
    TypeF64(Option<f64>),
    TypeMat4(Option<guest::Mat4>),
    TypeI32(Option<i32>),
    TypeQuat(Option<guest::Quat>),
    TypeString(Option<&'a str>),
    TypeU32(Option<u32>),
    TypeU64(Option<u64>),
    TypeVec2(Option<guest::Vec2>),
    TypeVec3(Option<guest::Vec3>),
    TypeVec4(Option<guest::Vec4>),
    TypeUVec2(Option<guest::Uvec2>),
    TypeUVec3(Option<guest::Uvec3>),
    TypeUVec4(Option<guest::Uvec4>),
}
impl<'a> ComponentOptionType<'a> {
    pub fn as_guest(&self) -> guest::ComponentOptionType<'a> {
        match self {
            Self::TypeEmpty(v) => guest::ComponentOptionType::TypeEmpty(*v),
            Self::TypeBool(v) => guest::ComponentOptionType::TypeBool(*v),
            Self::TypeEntityId(v) => guest::ComponentOptionType::TypeEntityId(*v),
            Self::TypeF32(v) => guest::ComponentOptionType::TypeF32(*v),
            Self::TypeF64(v) => guest::ComponentOptionType::TypeF64(*v),
            Self::TypeMat4(v) => guest::ComponentOptionType::TypeMat4(*v),
            Self::TypeI32(v) => guest::ComponentOptionType::TypeI32(*v),
            Self::TypeQuat(v) => guest::ComponentOptionType::TypeQuat(*v),
            Self::TypeString(v) => guest::ComponentOptionType::TypeString(*v),
            Self::TypeU32(v) => guest::ComponentOptionType::TypeU32(*v),
            Self::TypeU64(v) => guest::ComponentOptionType::TypeU64(*v),
            Self::TypeVec2(v) => guest::ComponentOptionType::TypeVec2(*v),
            Self::TypeVec3(v) => guest::ComponentOptionType::TypeVec3(*v),
            Self::TypeVec4(v) => guest::ComponentOptionType::TypeVec4(*v),
            Self::TypeUVec2(v) => guest::ComponentOptionType::TypeUvec2(*v),
            Self::TypeUVec3(v) => guest::ComponentOptionType::TypeUvec3(*v),
            Self::TypeUVec4(v) => guest::ComponentOptionType::TypeUvec4(*v),
        }
    }
}

pub(crate) enum ComponentType<'a> {
    TypeEmpty(()),
    TypeBool(bool),
    TypeEntityId(guest::EntityId),
    TypeF32(f32),
    TypeF64(f64),
    TypeMat4(guest::Mat4),
    TypeI32(i32),
    TypeQuat(guest::Quat),
    TypeString(String),
    TypeU32(u32),
    TypeU64(u64),
    TypeVec2(guest::Vec2),
    TypeVec3(guest::Vec3),
    TypeVec4(guest::Vec4),
    TypeUVec2(guest::Uvec2),
    TypeUVec3(guest::Uvec3),
    TypeUVec4(guest::Uvec4),
    TypeList(ComponentListType<'a>),
    TypeOption(ComponentOptionType<'a>),
}
impl<'a> ComponentType<'a> {
    pub fn as_guest(&'a self) -> guest::ComponentType<'a> {
        match self {
            Self::TypeEmpty(_) => guest::ComponentType::TypeEmpty(()),
            Self::TypeBool(v) => guest::ComponentType::TypeBool(*v),
            Self::TypeEntityId(v) => guest::ComponentType::TypeEntityId(*v),
            Self::TypeF32(v) => guest::ComponentType::TypeF32(*v),
            Self::TypeF64(v) => guest::ComponentType::TypeF64(*v),
            Self::TypeMat4(v) => guest::ComponentType::TypeMat4(*v),
            Self::TypeI32(v) => guest::ComponentType::TypeI32(*v),
            Self::TypeQuat(v) => guest::ComponentType::TypeQuat(*v),
            Self::TypeString(v) => guest::ComponentType::TypeString(v),
            Self::TypeU32(v) => guest::ComponentType::TypeU32(*v),
            Self::TypeU64(v) => guest::ComponentType::TypeU64(*v),
            Self::TypeVec2(v) => guest::ComponentType::TypeVec2(*v),
            Self::TypeVec3(v) => guest::ComponentType::TypeVec3(*v),
            Self::TypeVec4(v) => guest::ComponentType::TypeVec4(*v),
            Self::TypeUVec2(v) => guest::ComponentType::TypeUvec2(*v),
            Self::TypeUVec3(v) => guest::ComponentType::TypeUvec3(*v),
            Self::TypeUVec4(v) => guest::ComponentType::TypeUvec4(*v),
            Self::TypeList(v) => guest::ComponentType::TypeList(v.as_guest()),
            Self::TypeOption(v) => guest::ComponentType::TypeOption(v.as_guest()),
        }
    }
}

impl<'a> GuestConvert for &'a host::ComponentListTypeResult {
    type Item = ComponentListType<'a>;
    fn guest_convert(self) -> Self::Item {
        type S = host::ComponentListTypeResult;
        match self {
            S::TypeEmpty(c) => Self::Item::TypeEmpty(c.iter().map(|s| s.guest_convert()).collect()),
            S::TypeBool(c) => Self::Item::TypeBool(c.iter().map(|s| s.guest_convert()).collect()),
            S::TypeEntityId(c) => {
                Self::Item::TypeEntityId(c.iter().map(|s| s.guest_convert()).collect())
            }
            S::TypeF32(c) => Self::Item::TypeF32(c.iter().map(|s| s.guest_convert()).collect()),
            S::TypeF64(c) => Self::Item::TypeF64(c.iter().map(|s| s.guest_convert()).collect()),
            S::TypeMat4(c) => Self::Item::TypeMat4(c.iter().map(|s| s.guest_convert()).collect()),
            S::TypeI32(c) => Self::Item::TypeI32(c.iter().map(|s| s.guest_convert()).collect()),
            S::TypeQuat(c) => Self::Item::TypeQuat(c.iter().map(|s| s.guest_convert()).collect()),
            S::TypeString(c) => Self::Item::TypeString(c.iter().map(|s| s.as_str()).collect()),
            S::TypeU32(c) => Self::Item::TypeU32(c.iter().map(|s| s.guest_convert()).collect()),
            S::TypeU64(c) => Self::Item::TypeU64(c.iter().map(|s| s.guest_convert()).collect()),
            S::TypeVec2(c) => Self::Item::TypeVec2(c.iter().map(|s| s.guest_convert()).collect()),
            S::TypeVec3(c) => Self::Item::TypeVec3(c.iter().map(|s| s.guest_convert()).collect()),
            S::TypeVec4(c) => Self::Item::TypeVec4(c.iter().map(|s| s.guest_convert()).collect()),
            S::TypeUvec2(c) => Self::Item::TypeUVec2(c.iter().map(|s| s.guest_convert()).collect()),
            S::TypeUvec3(c) => Self::Item::TypeUVec3(c.iter().map(|s| s.guest_convert()).collect()),
            S::TypeUvec4(c) => Self::Item::TypeUVec4(c.iter().map(|s| s.guest_convert()).collect()),
        }
    }
}

impl<'a> GuestConvert for &'a host::ComponentOptionTypeResult {
    type Item = ComponentOptionType<'a>;
    fn guest_convert(self) -> Self::Item {
        type S = host::ComponentOptionTypeResult;
        match self {
            S::TypeEmpty(c) => Self::Item::TypeEmpty(c.guest_convert()),
            S::TypeBool(c) => Self::Item::TypeBool(c.guest_convert()),
            S::TypeEntityId(c) => Self::Item::TypeEntityId(c.guest_convert()),
            S::TypeF32(c) => Self::Item::TypeF32(c.guest_convert()),
            S::TypeF64(c) => Self::Item::TypeF64(c.guest_convert()),
            S::TypeMat4(c) => Self::Item::TypeMat4(c.guest_convert()),
            S::TypeI32(c) => Self::Item::TypeI32(c.guest_convert()),
            S::TypeQuat(c) => Self::Item::TypeQuat(c.guest_convert()),
            S::TypeString(c) => Self::Item::TypeString(c.as_deref()),
            S::TypeU32(c) => Self::Item::TypeU32(c.guest_convert()),
            S::TypeU64(c) => Self::Item::TypeU64(c.guest_convert()),
            S::TypeVec2(c) => Self::Item::TypeVec2(c.guest_convert()),
            S::TypeVec3(c) => Self::Item::TypeVec3(c.guest_convert()),
            S::TypeVec4(c) => Self::Item::TypeVec4(c.guest_convert()),
            S::TypeUvec2(c) => Self::Item::TypeUVec2(c.guest_convert()),
            S::TypeUvec3(c) => Self::Item::TypeUVec3(c.guest_convert()),
            S::TypeUvec4(c) => Self::Item::TypeUVec4(c.guest_convert()),
        }
    }
}

impl<'a> GuestConvert for &'a host::ComponentTypeResult {
    type Item = ComponentType<'a>;
    fn guest_convert(self) -> Self::Item {
        type S = host::ComponentTypeResult;
        match self {
            S::TypeEmpty(c) => {
                c.guest_convert();
                Self::Item::TypeEmpty(())
            }
            S::TypeBool(c) => Self::Item::TypeBool(c.guest_convert()),
            S::TypeEntityId(c) => Self::Item::TypeEntityId(c.guest_convert()),
            S::TypeF32(c) => Self::Item::TypeF32(c.guest_convert()),
            S::TypeF64(c) => Self::Item::TypeF64(c.guest_convert()),
            S::TypeMat4(c) => Self::Item::TypeMat4(c.guest_convert()),
            S::TypeI32(c) => Self::Item::TypeI32(c.guest_convert()),
            S::TypeQuat(c) => Self::Item::TypeQuat(c.guest_convert()),
            S::TypeString(c) => Self::Item::TypeString(c.to_owned()),
            S::TypeU32(c) => Self::Item::TypeU32(c.guest_convert()),
            S::TypeU64(c) => Self::Item::TypeU64(c.guest_convert()),
            S::TypeVec2(c) => Self::Item::TypeVec2(c.guest_convert()),
            S::TypeVec3(c) => Self::Item::TypeVec3(c.guest_convert()),
            S::TypeVec4(c) => Self::Item::TypeVec4(c.guest_convert()),
            S::TypeUvec2(c) => Self::Item::TypeUVec2(c.guest_convert()),
            S::TypeUvec3(c) => Self::Item::TypeUVec3(c.guest_convert()),
            S::TypeUvec4(c) => Self::Item::TypeUVec4(c.guest_convert()),
            S::TypeList(c) => Self::Item::TypeList(c.guest_convert()),
            S::TypeOption(c) => Self::Item::TypeOption(c.guest_convert()),
        }
    }
}
