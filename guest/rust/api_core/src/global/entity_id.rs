use crate::internal::{
    conversion::{FromBindgen, IntoBindgen},
    wit,
};

use data_encoding::BASE64URL_NOPAD as BASE64;

/// An identifier for an entity in the world.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct EntityId {
    #[doc(hidden)]
    pub id0: u64,
    #[doc(hidden)]
    pub id1: u64,
}
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

        Self {
            id0: u64::from_le_bytes(bytes[0..8].try_into().unwrap()),
            id1: u64::from_le_bytes(bytes[8..].try_into().unwrap()),
        }
    }

    /// Convert this [EntityId] to a Base64 representation.
    pub fn to_base64(&self) -> String {
        let mut bytes = [0u8; 16];
        bytes[0..8].copy_from_slice(&self.id0.to_le_bytes());
        bytes[8..].copy_from_slice(&self.id1.to_le_bytes());

        BASE64.encode(&bytes[..])
    }

    /// Return a null [EntityId]
    pub const fn null() -> Self {
        Self { id0: 0, id1: 0 }
    }
    /// Returns true if this is a null [EntityId]
    pub const fn is_null(&self) -> bool {
        self.id0 == 0 && self.id1 == 0
    }
    /// Return an [EntityId] pointing to the resources entity
    pub fn resources() -> Self {
        wit::entity::resources().from_bindgen()
    }
    /// Returns true if this is pointing to a resources entity
    pub fn is_resources(&self) -> bool {
        *self == Self::resources()
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
        wit::types::EntityId {
            id0: self.id0,
            id1: self.id1,
        }
    }
}
impl FromBindgen for wit::types::EntityId {
    type Item = EntityId;
    fn from_bindgen(self) -> Self::Item {
        EntityId {
            id0: self.id0,
            id1: self.id1,
        }
    }
}
