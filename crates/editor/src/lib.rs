use kiwi_ecs::{components, EntityUid};
use std::iter::Cloned;

#[macro_use]
extern crate closure;
pub mod intents;
pub mod rpc;
pub mod ui;

components!("editor", {
    selection: Selection,
    prev_selection: Selection,
});

pub fn init_all_components() {
    init_components();
    intents::init_components();
}

pub const GRID_SIZE: f32 = 1.0;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Selection {
    pub entities: Vec<EntityUid>,
}

impl<'a> IntoIterator for &'a Selection {
    type Item = EntityUid;

    type IntoIter = Cloned<std::slice::Iter<'a, EntityUid>>;

    fn into_iter(self) -> Self::IntoIter {
        self.entities.iter().cloned()
    }
}

impl FromIterator<EntityUid> for Selection {
    fn from_iter<T: IntoIterator<Item = EntityUid>>(iter: T) -> Self {
        Self::new(iter.into_iter().collect::<Vec<_>>())
    }
}

impl Selection {
    pub fn new(entities: impl Into<Vec<EntityUid>>) -> Selection {
        Self { entities: entities.into() }
    }

    pub fn iter(&self) -> Cloned<std::slice::Iter<EntityUid>> {
        self.entities.iter().cloned()
    }

    pub fn contains(&self, id: &EntityUid) -> bool {
        self.entities.iter().any(|v| v == id)
    }

    /// idempotent
    pub fn add(&mut self, id: EntityUid) {
        if self.contains(&id) {
            return;
        }
        self.entities.push(id)
    }

    pub fn remove(&mut self, id: &EntityUid) {
        self.entities.retain(|v| v != id)
    }

    pub fn toggle(&mut self, id: EntityUid) {
        if self.contains(&id) {
            self.remove(&id)
        } else {
            self.entities.push(id)
        }
    }

    pub fn union(&mut self, other: &Self) {
        for id in &other.entities {
            self.add(id.clone())
        }
    }

    pub fn difference(&mut self, other: &Self) {
        for id in &other.entities {
            self.remove(id)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn clear(&mut self) {
        self.entities.clear()
    }
}
