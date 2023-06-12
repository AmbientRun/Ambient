use std::{
    cmp::Ordering,
    collections::{btree_set::Range, BTreeSet, HashMap},
    fmt::Debug,
    ops::RangeBounds,
};

use itertools::Itertools;

use crate::{
    ArchetypeFilter, Component, ComponentDesc, ComponentEntry, ComponentValue, EntityId, FnSystem,
    Query, SystemGroup, World,
};

#[derive(Clone)]
pub struct IndexColumns {
    comparators: Vec<fn(&ComponentEntry, &ComponentEntry) -> Ordering>,
    components: Vec<ComponentDesc>,
}

impl Debug for IndexColumns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IndexColumns")
            .field("comparators", &self.comparators.len())
            .field("components", &self.components)
            .finish()
    }
}

impl IndexColumns {
    pub fn new() -> Self {
        Self {
            comparators: Default::default(),
            components: Default::default(),
        }
    }

    pub fn add_column<T: ComponentValue + Ord>(mut self, component: Component<T>) -> Self {
        self.comparators
            .push(|a, b| a.downcast_ref::<T>().cmp(b.downcast_ref::<T>()));
        self.components.push(component.desc());
        self
    }

    pub fn key_from_entity(&self, world: &World, entity: EntityId) -> IndexKey {
        self.try_key_from_entity(world, entity).unwrap()
    }

    pub fn try_key_from_entity(&self, world: &World, entity: EntityId) -> Option<IndexKey> {
        let fields = self
            .components
            .iter()
            .zip(self.comparators.iter())
            .map(|(&component, &comparator)| {
                let value = world.get_entry(entity, component).ok()?;
                Some(IndexField::Exact(IndexFieldValue { comparator, value }))
            })
            .collect::<Option<Vec<_>>>()?;

        Some(IndexKey {
            fields,
            id: IndexIdField::Exact(entity),
        })
    }
}

/// An ECS entity index
///
/// Can be used directly, or use `index_system`
#[derive(Clone, Debug)]
pub struct Index {
    columns: IndexColumns,
    index: BTreeSet<IndexKey>,
    ids_to_keys: HashMap<EntityId, IndexKey>,
}

impl Index {
    pub fn new(columns: IndexColumns) -> Self {
        Self {
            columns,
            index: Default::default(),
            ids_to_keys: Default::default(),
        }
    }
    pub fn insert_entity(&mut self, world: &World, id: EntityId) {
        assert!(!id.is_null());
        self.insert(self.columns.key_from_entity(world, id));
    }
    pub fn insert(&mut self, key: IndexKey) {
        self.ids_to_keys.insert(
            key.id
                .id()
                .expect("Must use IndexKey::exact when inserting"),
            key.clone(),
        );
        self.index.insert(key);
    }

    pub fn remove(&mut self, id: EntityId) -> bool {
        if let Some(key) = self.ids_to_keys.remove(&id) {
            self.index.remove(&key)
        } else {
            false
        }
    }

    /// Query the set between the specified ordered range
    pub fn range<R>(&self, range: R) -> Range<'_, IndexKey>
    where
        R: RangeBounds<IndexKey>,
    {
        self.index.range(range)
    }
}
impl std::fmt::Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Index")
            .field("columns", &self.columns)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
// Why the entity id is inside the IndexKey:
//
// First of all, this is to make two entity entries in the Index distinctit, even
// if the "fields" are the same.
//
// To query, you then use IndexIdField::Min and IndexIdField::Max to specify
// that you want to go from the start of the range of entity ids with the same
// value (or "fields").
pub struct IndexKey {
    pub fields: Vec<IndexField>,
    pub id: IndexIdField,
}

impl IndexKey {
    pub fn min(fields: Vec<IndexField>) -> Self {
        Self {
            fields,
            id: IndexIdField::Min,
        }
    }
    pub fn max(fields: Vec<IndexField>) -> Self {
        Self {
            fields,
            id: IndexIdField::Max,
        }
    }
    pub fn exact(fields: Vec<IndexField>, id: EntityId) -> Self {
        Self {
            fields,
            id: IndexIdField::Exact(id),
        }
    }
    pub fn id(&self) -> Option<EntityId> {
        self.id.id()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum IndexIdField {
    Min,
    Exact(EntityId),
    Max,
}
impl IndexIdField {
    pub fn id(&self) -> Option<EntityId> {
        if let Self::Exact(id) = self {
            Some(*id)
        } else {
            None
        }
    }
}

// https://stackoverflow.com/questions/70497455/filtering-querying-a-multi-key-btree-index-in-rust
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum IndexField {
    Min,
    Exact(IndexFieldValue),
    Max,
}
impl IndexField {
    pub fn exact<T: ComponentValue + Ord>(component: Component<T>, value: T) -> Self {
        Self::Exact(IndexFieldValue::new(component, value))
    }
}
#[derive(Clone)]
pub struct IndexFieldValue {
    comparator: fn(&ComponentEntry, &ComponentEntry) -> Ordering,
    value: ComponentEntry,
}
impl IndexFieldValue {
    pub fn new<T: ComponentValue + Ord>(component: Component<T>, value: T) -> Self {
        Self {
            comparator: |a, b| a.downcast_ref::<T>().cmp(b.downcast_ref::<T>()),
            value: ComponentEntry::new(component, value),
        }
    }
}
impl Debug for IndexFieldValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IndexFieldValue")
            .field("value", &self.value)
            .finish()
    }
}
impl PartialEq for IndexFieldValue {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}
impl Eq for IndexFieldValue {}
impl PartialOrd for IndexFieldValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for IndexFieldValue {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.comparator)(&self.value, &other.value)
    }
}

/// Creates and maintains an ECS Index as a resource on the world
pub fn index_system(
    mut filter: ArchetypeFilter,
    columns: IndexColumns,
    index_resource: Component<Index>,
) -> SystemGroup {
    for &c in &columns.components {
        filter = filter.incl_ref(c);
    }
    let components = columns.components.clone();
    SystemGroup::new(
        "index_system",
        vec![
            Box::new(FnSystem::new(move |world, _| {
                if !world.has_component(world.resource_entity(), index_resource) {
                    world.add_resource(index_resource, Index::new(columns.clone()));
                }
            })),
            Query::new(filter.clone())
                .spawned()
                .to_system(move |q, world, qs, _| {
                    let keys = {
                        let index = world.resource(index_resource);
                        q.iter(world, Some(qs))
                            .map(|x| index.columns.key_from_entity(world, x.id()))
                            .collect_vec()
                    };
                    let index = world.resource_mut(index_resource);
                    for key in keys {
                        index.insert(key);
                    }
                }),
            Query::new(filter.clone())
                .despawned()
                .to_system(move |q, world, qs, _| {
                    let ids = q.iter(world, Some(qs)).map(|x| x.id()).collect_vec();
                    let index = world.resource_mut(index_resource);
                    for id in ids {
                        index.remove(id);
                    }
                }),
            Query::any_changed(components)
                .filter(&filter)
                .to_system(move |q, world, qs, _| {
                    let keys = {
                        let index = world.resource(index_resource);
                        q.iter(world, Some(qs))
                            .map(|x| index.columns.key_from_entity(world, x.id()))
                            .collect_vec()
                    };
                    let index = world.resource_mut(index_resource);
                    for key in keys {
                        index.remove(key.id().unwrap());
                        index.insert(key);
                    }
                }),
        ],
    )
}

pub trait IndexExt {
    fn sync_index(&mut self, index: Component<Index>, id: EntityId, filter: ArchetypeFilter);
}

impl IndexExt for World {
    fn sync_index(&mut self, index: Component<Index>, id: EntityId, filter: ArchetypeFilter) {
        let key = { self.resource(index).columns.try_key_from_entity(self, id) };

        let matches = filter.matches_entity(self, id);
        let index = self.resource_mut(index);
        index.remove(id);

        if matches {
            if let Some(key) = key {
                index.insert(key)
            }
        }
    }
}
