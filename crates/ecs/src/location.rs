use std::{
    fmt::{self, Debug}, hash::{BuildHasher, Hasher}, str::FromStr
};

use data_encoding::BASE64URL_NOPAD;
use serde::{
    de::{self, Visitor}, Deserialize, Deserializer, Serialize, Serializer
};

#[derive(Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct EntityId(pub u128);
impl EntityId {
    pub fn new() -> Self {
        Self(rand::random())
    }
    pub fn null() -> Self {
        Self(0)
    }
    pub fn is_null(&self) -> bool {
        self.0 == 0
    }
    pub fn resources() -> Self {
        Self(1)
    }
    pub fn is_resources(&self) -> bool {
        self.0 == 1
    }
    pub fn from_u64s(a: u64, b: u64) -> Self {
        let bytes = [a.to_le_bytes(), b.to_le_bytes()].concat();
        Self(u128::from_le_bytes(bytes.try_into().unwrap()))
    }
    pub fn to_u64s(&self) -> (u64, u64) {
        let bytes: [u8; 16] = self.0.to_le_bytes();
        (u64::from_le_bytes(bytes[0..8].try_into().unwrap()), u64::from_le_bytes(bytes[8..].try_into().unwrap()))
    }
    pub fn to_base64(&self) -> String {
        BASE64URL_NOPAD.encode(&self.0.to_le_bytes())
    }
    pub fn from_base64(value: &str) -> Result<Self, data_encoding::DecodeError> {
        let bytes = BASE64URL_NOPAD.decode(value.as_bytes())?;
        Ok(Self(u128::from_le_bytes(bytes.try_into().unwrap())))
    }
}
impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_base64())
    }
}
impl std::fmt::Debug for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EntityId({}, {})", self.to_base64(), self.0)
    }
}
impl Default for EntityId {
    fn default() -> Self {
        Self::null()
    }
}
impl FromStr for EntityId {
    type Err = data_encoding::DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        EntityId::from_base64(s)
    }
}
impl Serialize for EntityId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_base64())
    }
}
impl<'de> Deserialize<'de> for EntityId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EntityIdVisitor;

        impl<'de> Visitor<'de> for EntityIdVisitor {
            type Value = EntityId;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct EntityId")
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                EntityId::from_base64(v).map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_str(EntityIdVisitor)
    }
}

#[test]
fn test_entity_id_bytes() {
    for _ in 0..100 {
        let id = EntityId::new();
        let (a, b) = id.to_u64s();
        assert_eq!(id, EntityId::from_u64s(a, b));
    }
    let id = EntityId(u128::MAX);
    let (a, b) = id.to_u64s();
    assert_eq!(id, EntityId::from_u64s(a, b));
    let id = EntityId(u128::MIN);
    let (a, b) = id.to_u64s();
    assert_eq!(id, EntityId::from_u64s(a, b));
}
#[test]
fn test_entity_id_serialization() {
    for _ in 0..100 {
        let id = EntityId::new();
        assert_eq!(id, serde_json::from_str(&serde_json::to_string(&id).unwrap()).unwrap());
    }
}

/// This just pipes a u64 value through
pub struct EntityIdHasher(u64);
impl Hasher for EntityIdHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, _bytes: &[u8]) {
        unreachable!()
    }

    fn write_u8(&mut self, _i: u8) {
        unreachable!()
    }
    fn write_u16(&mut self, _i: u16) {
        unreachable!()
    }
    fn write_u32(&mut self, _i: u32) {
        unreachable!();
    }
    fn write_u64(&mut self, _i: u64) {
        unreachable!()
    }
    fn write_u128(&mut self, i: u128) {
        self.0 = i as u64;
    }
    fn write_usize(&mut self, _i: usize) {
        unreachable!()
    }
    fn write_i8(&mut self, _i: i8) {
        unreachable!()
    }
    fn write_i16(&mut self, _i: i16) {
        unreachable!()
    }
    fn write_i32(&mut self, _i: i32) {
        unreachable!()
    }
    fn write_i64(&mut self, _i: i64) {
        unreachable!()
    }
    fn write_i128(&mut self, _i: i128) {
        unreachable!()
    }
    fn write_isize(&mut self, _i: isize) {
        unreachable!()
    }
}
#[derive(Clone)]
pub struct EntityIdHashBuilder;
impl BuildHasher for EntityIdHashBuilder {
    type Hasher = EntityIdHasher;
    fn build_hasher(&self) -> Self::Hasher {
        EntityIdHasher(0)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EntityLocation {
    pub archetype: usize,
    pub index: usize,
    pub gen: i32,
}
impl EntityLocation {
    pub(crate) fn empty() -> Self {
        Self { archetype: 0, index: 0, gen: -2 }
    }
}
