use crate::internal::{
    conversion::{FromBindgen, IntoBindgen},
    wit,
};

use data_encoding::BASE64URL_NOPAD as BASE64;
use serde::{Deserialize, Serialize};

/// An identifier for an entity in the world.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Serialize, Deserialize)]
pub struct EntityId(#[doc(hidden)] pub u128);
impl EntityId {
    /// Create a new [EntityId] from a Base64 representation.
    ///
    /// # Panics
    /// This will panic if
    /// - the Base64 string is more than 16 bytes
    /// - the Base64 string is not a valid EntityId.
    pub fn from_base64(encoded: &str) -> Self {
        let len = BASE64.decode_len(encoded.len()).unwrap();

        if len > 16 {
            panic!("base64 EntityId length {len} > 16");
        }

        let mut bytes = [0u8; 16];
        BASE64.decode_mut(encoded.as_bytes(), &mut bytes).unwrap();

        Self(u128::from_le_bytes(bytes))
    }

    /// Convert this [EntityId] to a Base64 representation.
    pub fn to_base64(&self) -> String {
        BASE64.encode(&self.0.to_le_bytes())
    }

    /// Return a null [EntityId]
    pub const fn null() -> Self {
        Self(0)
    }
    /// Returns true if this is a null [EntityId]
    pub const fn is_null(&self) -> bool {
        self.0 == 0
    }
    /// Return an [EntityId] pointing to the resources entity
    pub fn resources() -> Self {
        wit::entity::resources().from_bindgen()
    }
    /// Returns true if this is pointing to a resources entity
    pub fn is_resources(&self) -> bool {
        *self == Self::resources()
    }

    pub(crate) fn from_u64s(a: u64, b: u64) -> Self {
        let bytes = [a.to_le_bytes(), b.to_le_bytes()].concat();
        Self(u128::from_le_bytes(bytes.try_into().unwrap()))
    }
    pub(crate) fn to_u64s(self) -> (u64, u64) {
        let bytes: [u8; 16] = self.0.to_le_bytes();
        (
            u64::from_le_bytes(bytes[0..8].try_into().unwrap()),
            u64::from_le_bytes(bytes[8..].try_into().unwrap()),
        )
    }
}
impl Default for EntityId {
    fn default() -> Self {
        Self::null()
    }
}
impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_base64())
    }
}
impl std::fmt::Debug for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
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

impl FromBindgen for wit::types::Empty {
    type Item = ();

    fn from_bindgen(self) -> Self::Item {}
}

impl IntoBindgen for () {
    type Item = wit::types::Empty;

    fn into_bindgen(self) -> Self::Item {
        wit::types::Empty { dummy: 0 }
    }
}
