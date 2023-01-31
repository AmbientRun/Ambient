use std::{
    fmt::{self, Debug}, hash::{BuildHasher, Hasher}, num::ParseIntError, str::FromStr
};

use derive_more::Display;
use itertools::Itertools;
use serde::{
    de::{self, Visitor}, Deserialize, Deserializer, Serialize, Serializer
};

#[derive(Debug, Clone, Copy, Display, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
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
        Self(((a as u128) << 64) + b as u128)
    }
    pub fn to_u64s(&self) -> (u64, u64) {
        let high_byte: u64 = (self.0 >> 64) as u64;
        let low_byte: u64 = (self.0 & 0xffffffff) as u64;
        (high_byte, low_byte)
    }
}
impl Default for EntityId {
    fn default() -> Self {
        Self::null()
    }
}
impl FromStr for EntityId {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(EntityId(s.parse()?))
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
    fn write_u32(&mut self, i: u32) {
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
