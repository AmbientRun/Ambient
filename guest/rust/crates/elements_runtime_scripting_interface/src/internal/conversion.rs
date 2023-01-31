use crate::{host as sif, EntityId, Vec2};

/// Converts from a Rust representation to a wit-bindgen representation.
pub trait IntoBindgen {
    type Item;
    fn into_bindgen(self) -> Self::Item;
}

/// Converts from a wit-bindgen representation to a Rust representation.
pub trait FromBindgen {
    type Item;

    #[allow(clippy::wrong_self_convention)]
    fn from_bindgen(self) -> Self::Item;
}

impl IntoBindgen for EntityId {
    type Item = sif::EntityId;
    fn into_bindgen(self) -> Self::Item {
        sif::EntityId {
            id0: self.id0,
            id1: self.id1,
        }
    }
}
impl FromBindgen for sif::EntityId {
    type Item = EntityId;
    fn from_bindgen(self) -> Self::Item {
        EntityId {
            id0: self.id0,
            id1: self.id1,
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

impl<T> FromBindgen for Option<T>
where
    T: FromBindgen,
{
    type Item = Option<T::Item>;
    fn from_bindgen(self) -> Self::Item {
        self.map(|i| i.from_bindgen())
    }
}
