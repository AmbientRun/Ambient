use std::time::SystemTime;

use elements_animation as ea;
use elements_ecs::EntityId;
use elements_ecs::EntityUid;
use elements_std::asset_url::ObjectRef;
use elements_std::asset_url::TypedAssetUrl;
use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
use wit_bindgen_host_wasmtime_rust::{Endian, Le};

use super::interface as sif;

/// Converts from a Rust representation to a wit-bindgen representation.
pub trait IntoBindgen {
    type Item;
    fn into_bindgen(self) -> Self::Item;
}

/// Converts from a wit-bindgen representation to a Rust representation.
#[allow(clippy::wrong_self_convention)]
pub trait FromBindgen {
    type Item;
    fn from_bindgen(self) -> Self::Item;
}

impl IntoBindgen for EntityId {
    type Item = sif::EntityId;
    fn into_bindgen(self) -> Self::Item {
        sif::EntityId {
            namespace: self.namespace,
            id: self.id as u64,
            gen: self.gen,
        }
    }
}
impl FromBindgen for sif::EntityId {
    type Item = EntityId;
    fn from_bindgen(self) -> Self::Item {
        EntityId {
            namespace: self.namespace,
            id: self.id as usize,
            gen: self.gen,
        }
    }
}

impl IntoBindgen for Vec2 {
    type Item = sif::Vec2;
    fn into_bindgen(self) -> Self::Item {
        sif::Vec2 {
            x: self.x,
            y: self.y,
        }
    }
}
impl FromBindgen for sif::Vec2 {
    type Item = Vec2;
    fn from_bindgen(self) -> Self::Item {
        Vec2::new(self.x, self.y)
    }
}

impl IntoBindgen for Vec3 {
    type Item = sif::Vec3;
    fn into_bindgen(self) -> Self::Item {
        sif::Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}
impl FromBindgen for sif::Vec3 {
    type Item = Vec3;
    fn from_bindgen(self) -> Self::Item {
        Vec3::new(self.x, self.y, self.z)
    }
}

impl IntoBindgen for Vec4 {
    type Item = sif::Vec4;
    fn into_bindgen(self) -> Self::Item {
        sif::Vec4 {
            x: self.x,
            y: self.y,
            z: self.z,
            w: self.w,
        }
    }
}
impl FromBindgen for sif::Vec4 {
    type Item = Vec4;
    fn from_bindgen(self) -> Self::Item {
        Vec4::new(self.x, self.y, self.z, self.w)
    }
}

impl IntoBindgen for Quat {
    type Item = sif::Quat;
    fn into_bindgen(self) -> Self::Item {
        sif::Quat {
            x: self.x,
            y: self.y,
            z: self.z,
            w: self.w,
        }
    }
}
impl FromBindgen for sif::Quat {
    type Item = Quat;
    fn from_bindgen(self) -> Self::Item {
        Quat::from_array([self.x, self.y, self.z, self.w])
    }
}

impl IntoBindgen for Mat4 {
    type Item = sif::Mat4;
    fn into_bindgen(self) -> Self::Item {
        sif::Mat4 {
            x: self.x_axis.into_bindgen(),
            y: self.y_axis.into_bindgen(),
            z: self.z_axis.into_bindgen(),
            w: self.w_axis.into_bindgen(),
        }
    }
}
impl FromBindgen for sif::Mat4 {
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

impl IntoBindgen for ObjectRef {
    type Item = sif::ObjectRefResult;
    fn into_bindgen(self) -> Self::Item {
        Self::Item {
            id: self.to_string(),
        }
    }
}
impl<'a> FromBindgen for sif::ObjectRefParam<'a> {
    type Item = ObjectRef;
    fn from_bindgen(self) -> Self::Item {
        Self::Item::parse(self.id).unwrap()
    }
}
impl FromBindgen for sif::ObjectRefResult {
    type Item = ObjectRef;
    fn from_bindgen(self) -> Self::Item {
        Self::Item::parse(self.id).unwrap()
    }
}

impl IntoBindgen for EntityUid {
    type Item = sif::EntityUidResult;
    fn into_bindgen(self) -> Self::Item {
        Self::Item { id: self.0 }
    }
}
impl<'a> FromBindgen for sif::EntityUidParam<'a> {
    type Item = EntityUid;
    fn from_bindgen(self) -> Self::Item {
        EntityUid(self.id.to_owned())
    }
}
impl FromBindgen for sif::EntityUidResult {
    type Item = EntityUid;
    fn from_bindgen(self) -> Self::Item {
        EntityUid(self.id)
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
bindgen_passthrough!(i32);
bindgen_passthrough!(String);
bindgen_passthrough!(u32);
bindgen_passthrough!(u64);

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

impl<T> FromBindgen for Le<T>
where
    T: FromBindgen + Endian,
{
    type Item = T::Item;

    fn from_bindgen(self) -> Self::Item {
        self.get().from_bindgen()
    }
}

impl FromBindgen for sif::AnimationAction<'_> {
    type Item = ea::AnimationAction;
    fn from_bindgen(self) -> Self::Item {
        ea::AnimationAction {
            clip: ea::AnimationClipRef::FromModelAsset(
                TypedAssetUrl::parse(self.clip_url).unwrap(),
            ),
            time: ea::AnimationActionTime::Offset {
                start_time: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap(),
                speed: 1.0,
            },
            looping: self.looping,
            weight: self.weight,
        }
    }
}

impl FromBindgen for sif::AnimationController<'_> {
    type Item = ea::AnimationController;
    fn from_bindgen(self) -> Self::Item {
        ea::AnimationController {
            actions: self.actions.into_iter().map(|s| s.from_bindgen()).collect(),
            apply_base_pose: self.apply_base_pose,
        }
    }
}
