use std::time::Duration;

use ambient_ecs::EntityId;
use ambient_native_std::shapes::Ray;
use ambient_shared_types::{
    procedural_storage_handle_definitions, ProceduralMaterialHandle, ProceduralMeshHandle,
    ProceduralSamplerHandle, ProceduralTextureHandle,
};
use glam::{IVec2, IVec3, IVec4, Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
use paste::paste;
use ulid::Ulid;

use super::wit;

/// Converts from a Rust representation to a wit-bindgen representation.
///
/// Implemented on the Rust type to convert to a WIT type.
pub trait IntoBindgen {
    type Item;
    fn into_bindgen(self) -> Self::Item;
}

/// Converts from a wit-bindgen representation to a Rust representation.
///
/// Implemented on the WIT type to convert to a Rust type.
#[allow(clippy::wrong_self_convention)]
pub trait FromBindgen {
    type Item;
    fn from_bindgen(self) -> Self::Item;
}

impl IntoBindgen for EntityId {
    type Item = wit::types::EntityId;
    fn into_bindgen(self) -> Self::Item {
        let (id0, id1) = self.to_u64s();

        wit::types::EntityId { id0, id1 }
    }
}
impl FromBindgen for wit::types::EntityId {
    type Item = EntityId;
    fn from_bindgen(self) -> Self::Item {
        EntityId::from_u64s(self.id0, self.id1)
    }
}

impl IntoBindgen for Vec2 {
    type Item = wit::types::Vec2;
    fn into_bindgen(self) -> Self::Item {
        wit::types::Vec2 {
            x: self.x,
            y: self.y,
        }
    }
}
impl FromBindgen for wit::types::Vec2 {
    type Item = Vec2;
    fn from_bindgen(self) -> Self::Item {
        Vec2::new(self.x, self.y)
    }
}

impl IntoBindgen for Vec3 {
    type Item = wit::types::Vec3;
    fn into_bindgen(self) -> Self::Item {
        wit::types::Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}
impl FromBindgen for wit::types::Vec3 {
    type Item = Vec3;
    fn from_bindgen(self) -> Self::Item {
        Vec3::new(self.x, self.y, self.z)
    }
}

impl IntoBindgen for Vec4 {
    type Item = wit::types::Vec4;
    fn into_bindgen(self) -> Self::Item {
        wit::types::Vec4 {
            x: self.x,
            y: self.y,
            z: self.z,
            w: self.w,
        }
    }
}
impl FromBindgen for wit::types::Vec4 {
    type Item = Vec4;
    fn from_bindgen(self) -> Self::Item {
        Vec4::new(self.x, self.y, self.z, self.w)
    }
}

impl IntoBindgen for UVec2 {
    type Item = wit::types::Uvec2;
    fn into_bindgen(self) -> Self::Item {
        wit::types::Uvec2 {
            x: self.x,
            y: self.y,
        }
    }
}
impl FromBindgen for wit::types::Uvec2 {
    type Item = UVec2;
    fn from_bindgen(self) -> Self::Item {
        UVec2::new(self.x, self.y)
    }
}

impl IntoBindgen for UVec3 {
    type Item = wit::types::Uvec3;
    fn into_bindgen(self) -> Self::Item {
        wit::types::Uvec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}
impl FromBindgen for wit::types::Uvec3 {
    type Item = UVec3;
    fn from_bindgen(self) -> Self::Item {
        UVec3::new(self.x, self.y, self.z)
    }
}

impl IntoBindgen for UVec4 {
    type Item = wit::types::Uvec4;
    fn into_bindgen(self) -> Self::Item {
        wit::types::Uvec4 {
            x: self.x,
            y: self.y,
            z: self.z,
            w: self.w,
        }
    }
}
impl FromBindgen for wit::types::Uvec4 {
    type Item = UVec4;
    fn from_bindgen(self) -> Self::Item {
        UVec4::new(self.x, self.y, self.z, self.w)
    }
}

impl IntoBindgen for IVec2 {
    type Item = wit::types::Ivec2;
    fn into_bindgen(self) -> Self::Item {
        wit::types::Ivec2 {
            x: self.x,
            y: self.y,
        }
    }
}
impl FromBindgen for wit::types::Ivec2 {
    type Item = IVec2;
    fn from_bindgen(self) -> Self::Item {
        IVec2::new(self.x, self.y)
    }
}

impl IntoBindgen for IVec3 {
    type Item = wit::types::Ivec3;
    fn into_bindgen(self) -> Self::Item {
        wit::types::Ivec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}
impl FromBindgen for wit::types::Ivec3 {
    type Item = IVec3;
    fn from_bindgen(self) -> Self::Item {
        IVec3::new(self.x, self.y, self.z)
    }
}

impl IntoBindgen for IVec4 {
    type Item = wit::types::Ivec4;
    fn into_bindgen(self) -> Self::Item {
        wit::types::Ivec4 {
            x: self.x,
            y: self.y,
            z: self.z,
            w: self.w,
        }
    }
}
impl FromBindgen for wit::types::Ivec4 {
    type Item = IVec4;
    fn from_bindgen(self) -> Self::Item {
        IVec4::new(self.x, self.y, self.z, self.w)
    }
}

impl IntoBindgen for Quat {
    type Item = wit::types::Quat;
    fn into_bindgen(self) -> Self::Item {
        wit::types::Quat {
            x: self.x,
            y: self.y,
            z: self.z,
            w: self.w,
        }
    }
}
impl FromBindgen for wit::types::Quat {
    type Item = Quat;
    fn from_bindgen(self) -> Self::Item {
        Quat::from_array([self.x, self.y, self.z, self.w])
    }
}

impl IntoBindgen for Mat4 {
    type Item = wit::types::Mat4;
    fn into_bindgen(self) -> Self::Item {
        wit::types::Mat4 {
            x: self.x_axis.into_bindgen(),
            y: self.y_axis.into_bindgen(),
            z: self.z_axis.into_bindgen(),
            w: self.w_axis.into_bindgen(),
        }
    }
}
impl FromBindgen for wit::types::Mat4 {
    type Item = Mat4;
    fn from_bindgen(self) -> Self::Item {
        Mat4::from_cols(
            self.x.from_bindgen(),
            self.y.from_bindgen(),
            self.z.from_bindgen(),
            self.w.from_bindgen(),
        )
    }
}

impl IntoBindgen for Ray {
    type Item = wit::types::Ray;
    fn into_bindgen(self) -> Self::Item {
        wit::types::Ray {
            origin: self.origin.into_bindgen(),
            dir: self.dir.into_bindgen(),
        }
    }
}

impl FromBindgen for wit::types::Ray {
    type Item = Ray;
    fn from_bindgen(self) -> Self::Item {
        Ray {
            origin: self.origin.from_bindgen(),
            dir: self.dir.from_bindgen(),
        }
    }
}

impl IntoBindgen for Duration {
    type Item = wit::types::Duration;
    fn into_bindgen(self) -> Self::Item {
        wit::types::Duration {
            seconds: self.as_secs(),
            nanoseconds: self.subsec_nanos(),
        }
    }
}
impl FromBindgen for wit::types::Duration {
    type Item = Duration;
    fn from_bindgen(self) -> Self::Item {
        Duration::new(self.seconds, self.nanoseconds)
    }
}

macro_rules! bindgen_passthrough {
    ($type:ty) => {
        impl IntoBindgen for $type {
            type Item = Self;
            fn into_bindgen(self) -> Self::Item {
                self
            }
        }
        impl FromBindgen for $type {
            type Item = Self;
            fn from_bindgen(self) -> Self::Item {
                self
            }
        }
    };
}

bindgen_passthrough!(());
bindgen_passthrough!(bool);
bindgen_passthrough!(f32);
bindgen_passthrough!(f64);
bindgen_passthrough!(String);
bindgen_passthrough!(u8);
bindgen_passthrough!(u16);
bindgen_passthrough!(u32);
bindgen_passthrough!(u64);
bindgen_passthrough!(i8);
bindgen_passthrough!(i16);
bindgen_passthrough!(i32);
bindgen_passthrough!(i64);

impl<'a> FromBindgen for &'a str {
    type Item = String;
    fn from_bindgen(self) -> Self::Item {
        self.to_owned()
    }
}

impl<T> IntoBindgen for Option<T>
where
    T: IntoBindgen,
{
    type Item = Option<T::Item>;
    fn into_bindgen(self) -> Self::Item {
        self.map(|i| i.into_bindgen())
    }
}
impl<T> FromBindgen for Option<T>
where
    T: FromBindgen,
{
    type Item = Option<T::Item>;
    fn from_bindgen(self) -> Self::Item {
        self.map(|i| i.from_bindgen())
    }
}

impl<T> IntoBindgen for Vec<T>
where
    T: IntoBindgen,
{
    type Item = Vec<T::Item>;
    fn into_bindgen(self) -> Self::Item {
        self.into_iter().map(|i| i.into_bindgen()).collect()
    }
}
impl<T> FromBindgen for Vec<T>
where
    T: FromBindgen,
{
    type Item = Vec<T::Item>;
    fn from_bindgen(self) -> Self::Item {
        self.into_iter().map(|i| i.from_bindgen()).collect()
    }
}
impl<T> FromBindgen for &[T]
where
    T: FromBindgen + Clone,
{
    type Item = Vec<T::Item>;
    fn from_bindgen(self) -> Self::Item {
        self.iter().map(|i| i.clone().from_bindgen()).collect()
    }
}

impl FromBindgen for wit::types::Ulid {
    type Item = Ulid;

    fn from_bindgen(self) -> Self::Item {
        Ulid::from(self)
    }
}

impl IntoBindgen for Ulid {
    type Item = wit::types::Ulid;

    fn into_bindgen(self) -> Self::Item {
        self.into()
    }
}

macro_rules! make_procedural_storage_handle_converters {
    ($($name:ident),*) => { paste!{$(
        impl FromBindgen for wit::[<client_ $name>]::Handle {
            type Item = [<Procedural $name:camel Handle>];

            fn from_bindgen(self) -> Self::Item {
                Self::Item::from(self.ulid.from_bindgen())
            }
        }

        impl IntoBindgen for [<Procedural $name:camel Handle>] {
            type Item = wit::[<client_ $name>]::Handle;

            fn into_bindgen(self) -> Self::Item {
                Self::Item {
                    ulid: Ulid::from(self).into_bindgen(),
                }
            }
        }
    )*}};
}

procedural_storage_handle_definitions!(make_procedural_storage_handle_converters);

impl FromBindgen for wit::client_texture::Format {
    type Item = wgpu::TextureFormat;

    fn from_bindgen(self) -> Self::Item {
        match self {
            Self::R8Unorm => Self::Item::R8Unorm,
            Self::R8Snorm => Self::Item::R8Snorm,
            Self::R8Uint => Self::Item::R8Uint,
            Self::R8Sint => Self::Item::R8Sint,
            Self::R16Uint => Self::Item::R16Uint,
            Self::R16Sint => Self::Item::R16Sint,
            Self::R16Unorm => Self::Item::R16Unorm,
            Self::R16Snorm => Self::Item::R16Snorm,
            Self::R16Float => Self::Item::R16Float,
            Self::Rg8Unorm => Self::Item::Rg8Unorm,
            Self::Rg8Snorm => Self::Item::Rg8Snorm,
            Self::Rg8Uint => Self::Item::Rg8Uint,
            Self::Rg8Sint => Self::Item::Rg8Sint,
            Self::R32Uint => Self::Item::R32Uint,
            Self::R32Sint => Self::Item::R32Sint,
            Self::R32Float => Self::Item::R32Float,
            Self::Rg16Uint => Self::Item::Rg16Uint,
            Self::Rg16Sint => Self::Item::Rg16Sint,
            Self::Rg16Unorm => Self::Item::Rg16Unorm,
            Self::Rg16Snorm => Self::Item::Rg16Snorm,
            Self::Rg16Float => Self::Item::Rg16Float,
            Self::Rgba8Unorm => Self::Item::Rgba8Unorm,
            Self::Rgba8UnormSrgb => Self::Item::Rgba8UnormSrgb,
            Self::Rgba8Snorm => Self::Item::Rgba8Snorm,
            Self::Rgba8Uint => Self::Item::Rgba8Uint,
            Self::Rgba8Sint => Self::Item::Rgba8Sint,
            Self::Bgra8Unorm => Self::Item::Bgra8Unorm,
            Self::Bgra8UnormSrgb => Self::Item::Bgra8UnormSrgb,
            Self::Rgb9e5Ufloat => Self::Item::Rgb9e5Ufloat,
            Self::Rgb10a2Unorm => Self::Item::Rgb10a2Unorm,
            Self::Rg11b10Float => Self::Item::Rg11b10Float,
            Self::Rg32Uint => Self::Item::Rg32Uint,
            Self::Rg32Sint => Self::Item::Rg32Sint,
            Self::Rg32Float => Self::Item::Rg32Float,
            Self::Rgba16Uint => Self::Item::Rgba16Uint,
            Self::Rgba16Sint => Self::Item::Rgba16Sint,
            Self::Rgba16Unorm => Self::Item::Rgba16Unorm,
            Self::Rgba16Snorm => Self::Item::Rgba16Snorm,
            Self::Rgba16Float => Self::Item::Rgba16Float,
            Self::Rgba32Uint => Self::Item::Rgba32Uint,
            Self::Rgba32Sint => Self::Item::Rgba32Sint,
            Self::Rgba32Float => Self::Item::Rgba32Float,
        }
    }
}

impl FromBindgen for wit::client_sampler::FilterMode {
    type Item = wgpu::FilterMode;

    fn from_bindgen(self) -> Self::Item {
        match self {
            Self::Nearest => Self::Item::Nearest,
            Self::Linear => Self::Item::Linear,
        }
    }
}

impl FromBindgen for wit::client_sampler::AddressMode {
    type Item = wgpu::AddressMode;

    fn from_bindgen(self) -> Self::Item {
        match self {
            Self::ClampToEdge => Self::Item::ClampToEdge,
            Self::Repeat => Self::Item::Repeat,
            Self::MirrorRepeat => Self::Item::MirrorRepeat,
        }
    }
}
