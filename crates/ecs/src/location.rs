use std::{
    fmt::{self, Debug}, num::ParseIntError, str::FromStr
};

use derive_more::Display;
use itertools::Itertools;
use serde::{
    de::{self, Visitor}, Deserialize, Deserializer, Serialize, Serializer
};

#[derive(Debug, Clone, Copy, Display, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[display(fmt = "{namespace}:{id}:{gen}")]
pub struct EntityId {
    pub namespace: u8,
    pub id: usize,
    pub gen: i32,
}
impl EntityId {
    pub(super) fn new(namespace: u8, id: usize, gen: i32) -> Self {
        Self { namespace, id, gen }
    }
    pub fn null() -> Self {
        Self { namespace: 0, id: 0, gen: -1 }
    }
    pub fn is_null(&self) -> bool {
        self.gen == -1
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
        let ss: Vec<&str> = s.split(':').collect();

        Ok(EntityId { namespace: ss[0].parse()?, id: ss[1].parse()?, gen: ss[2].parse()? })
    }
}
impl Serialize for EntityId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
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
                v.parse().map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_str(EntityIdVisitor)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EntityLocation {
    pub archetype: usize,
    pub index: usize,
    pub gen: i32,
}
impl EntityLocation {
    fn empty() -> Self {
        Self { archetype: 0, index: 0, gen: -2 }
    }
    fn is_empty(&self) -> bool {
        self.gen < 0
    }
}

#[derive(Clone)]
pub(super) struct EntityLocations {
    local_namespace: u8,
    pub(super) allocated: Vec<Vec<EntityLocation>>,
    freed: Vec<EntityId>,
}
impl EntityLocations {
    pub(super) fn new(local_namespace: u8) -> Self {
        Self { local_namespace, allocated: vec![Vec::new(); 256], freed: Vec::new() }
    }
    pub(super) fn get(&self, id: EntityId) -> Option<EntityLocation> {
        if let Some(loc) = self.allocated[id.namespace as usize].get(id.id) {
            if loc.gen == id.gen {
                return Some(*loc);
            }
        }
        None
    }
    pub(super) fn get_mut(&mut self, id: EntityId) -> Option<&mut EntityLocation> {
        if let Some(loc) = self.allocated[id.namespace as usize].get_mut(id.id) {
            if loc.gen == id.gen {
                return Some(loc);
            }
        }
        None
    }
    pub(super) fn exists(&self, id: EntityId) -> bool {
        self.get(id).is_some()
    }
    pub(super) fn allocate(&mut self, count: usize) -> Vec<EntityId> {
        let freed = self.freed.split_off((self.freed.len() as i32 - count as i32).max(0) as usize);
        let mut freed_new = freed
            .into_iter()
            .map(|old_id| {
                let id = EntityId::new(self.local_namespace, old_id.id, old_id.gen + 1);
                self.allocated[self.local_namespace as usize][id.id] = EntityLocation { archetype: 0, index: 0, gen: id.gen };
                id
            })
            .collect_vec();
        if freed_new.len() == count {
            freed_new
        } else {
            let remaining_count = count - freed_new.len();
            let alloced = (0..remaining_count)
                .map(|i| EntityId::new(self.local_namespace, self.allocated[self.local_namespace as usize].len() + i, 0))
                .collect_vec();
            self.allocated[self.local_namespace as usize].extend(alloced.iter().map(|id| EntityLocation {
                archetype: 0,
                index: 0,
                gen: id.gen,
            }));
            freed_new.extend(alloced.into_iter());
            freed_new
        }
    }
    pub(super) fn allocate_mirror(&mut self, id: EntityId) -> bool {
        if self.allocated[id.namespace as usize].len() <= id.id {
            self.allocated[id.namespace as usize].resize_with(id.id + 1, EntityLocation::empty);
        }
        if !self.allocated[id.namespace as usize][id.id].is_empty() {
            return false;
        }
        self.allocated[id.namespace as usize][id.id] = EntityLocation { archetype: 0, index: 0, gen: id.gen };
        true
    }
    pub(super) fn free(&mut self, removed_entity_loc: EntityLocation, removed_id: EntityId, swapped_id: EntityId) {
        if removed_id.namespace == self.local_namespace {
            self.freed.push(removed_id);
        }
        self.allocated[removed_id.namespace as usize][removed_id.id] = EntityLocation::empty();
        if removed_id != swapped_id {
            self.on_swap_remove(removed_entity_loc, swapped_id);
        }
    }
    pub(super) fn on_swap_remove(&mut self, removed_entity_loc: EntityLocation, swapped_id: EntityId) {
        self.allocated[swapped_id.namespace as usize][swapped_id.id].index = removed_entity_loc.index;
    }
    #[allow(dead_code)]
    pub(super) fn namespace(&self) -> u8 {
        self.local_namespace
    }
    pub(super) fn set_namespace(&mut self, local_namespace: u8) {
        assert_ne!(self.local_namespace, local_namespace);
        self.local_namespace = local_namespace;
        self.freed.clear();
    }
    // fn on_archetype_removed(&mut self, arch_id: usize) {
    //     for loc in self.allocated.iter_mut() {
    //         if loc.archetype > arch_id {
    //             loc.archetype -= 1;
    //         }
    //     }
    // }
}

#[test]
fn test_mirror() {
    let mut locs = EntityLocations::new(0);
    let remote_id = EntityId { namespace: 1, id: 10, gen: 5 };
    locs.allocate_mirror(remote_id);
    assert!(locs.get(remote_id).is_some());
}
