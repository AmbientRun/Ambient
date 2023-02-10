use std::{borrow::Borrow, collections::HashMap, fmt::Display};

use kiwi_std::friendly_id;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{query, uid, EntityId, World};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EntityUid(pub String);
impl EntityUid {
    pub fn create() -> Self {
        Self(friendly_id())
    }
}
impl Display for EntityUid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub fn lookup_uid(world: &World, uid_search: &EntityUid) -> Option<EntityId> {
    query(uid()).iter(world, None).find_map(|(id, uid)| (uid == uid_search).then_some(id))
}

#[derive(Error, Debug, Clone)]
pub enum UidError {
    #[error("Failed to resolve uid: {0}")]
    InvalidUid(EntityUid),
}

#[derive(Default, PartialEq, Eq, Debug, Clone)]
pub struct UidLookup {
    inner: HashMap<EntityUid, EntityId>,
}

impl UidLookup {
    pub fn inner(&self) -> &HashMap<EntityUid, EntityId> {
        &self.inner
    }

    pub fn get<Q>(&self, k: &Q) -> Result<EntityId, UidError>
    where
        Q: ?Sized + std::hash::Hash + Eq + ToOwned<Owned = EntityUid>,
        EntityUid: Borrow<Q>,
    {
        self.inner.get(k).copied().ok_or_else(|| UidError::InvalidUid(k.to_owned()))
    }

    pub fn contains_key<Q>(&self, k: &Q) -> bool
    where
        Q: ?Sized + std::hash::Hash + Eq,
        EntityUid: Borrow<Q>,
    {
        self.inner.contains_key(k)
    }

    pub fn insert(&mut self, k: EntityUid, v: EntityId) {
        self.inner.insert(k, v);
    }

    pub fn remove<Q>(&mut self, k: &Q) -> Option<EntityId>
    where
        Q: ?Sized + std::hash::Hash + Eq,
        EntityUid: Borrow<Q>,
    {
        self.inner.remove(k)
    }
}

impl Extend<(EntityUid, EntityId)> for UidLookup {
    fn extend<T: IntoIterator<Item = (EntityUid, EntityId)>>(&mut self, iter: T) {
        self.inner.extend(iter)
    }
}
