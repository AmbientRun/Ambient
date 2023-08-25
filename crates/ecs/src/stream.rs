use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fmt::Display,
    sync::Arc,
};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use super::{
    ArchetypeFilter, Component, ComponentValue, Entity, EntityId, FramedEventsReader, Query,
    QueryState, World,
};
use crate::{ComponentDesc, ComponentEntry};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct WorldDiff {
    pub changes: Vec<WorldChange>,
}
impl WorldDiff {
    pub fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
    }
    pub fn set<T: ComponentValue>(self, id: EntityId, component: Component<T>, value: T) -> Self {
        self.set_entry(id, ComponentEntry::new(component, value))
    }
    pub fn add_component<T: ComponentValue>(
        self,
        id: EntityId,
        component: Component<T>,
        value: T,
    ) -> Self {
        self.add_entry(id, ComponentEntry::new(component, value))
    }

    pub fn remove_component(mut self, id: EntityId, component: ComponentDesc) -> Self {
        self.changes
            .push(WorldChange::RemoveComponents(id, vec![component]));
        self
    }

    pub fn remove_components_raw(mut self, id: EntityId, components: Vec<ComponentDesc>) -> Self {
        self.changes
            .push(WorldChange::RemoveComponents(id, components));
        self
    }
    pub fn set_entry(mut self, id: EntityId, entry: ComponentEntry) -> Self {
        self.changes
            .push(WorldChange::SetComponents(id, Entity::from(vec![entry])));
        self
    }
    pub fn add_entry(mut self, id: EntityId, entry: ComponentEntry) -> Self {
        let mut data = Entity::new();
        data.set_entry(entry);
        self.changes.push(WorldChange::AddComponents(id, data));
        self
    }
    pub fn despawn(mut self, ids: Vec<EntityId>) -> Self {
        self.changes
            .extend(ids.into_iter().map(WorldChange::Despawn));
        self
    }
    pub fn apply(self, world: &mut World, spawned_extra_data: Entity) {
        for change in self.changes.into_iter() {
            change.apply(world, &spawned_extra_data, false);
        }
    }
    pub fn is_empty(&self) -> bool {
        self.changes.len() == 0
    }
    /// This creates a list of changes that would take you from the `from` world to the `to` world, if applied to the `from` world.
    pub fn from_a_to_b(filter: WorldStreamFilter, from: &World, to: &World) -> Self {
        let from_entities: HashSet<EntityId> = filter.all_entities(from).collect();
        let to_entities: HashSet<EntityId> = filter.all_entities(to).collect();
        let spawned = to_entities
            .iter()
            .filter(|id| !from_entities.contains(id))
            .cloned()
            .collect_vec();
        let despawned = from_entities
            .iter()
            .filter(|id| !to_entities.contains(id))
            .cloned()
            .collect_vec();
        let in_both = to_entities
            .iter()
            .filter(|id| from_entities.contains(id))
            .cloned()
            .collect_vec();

        let spawned = spawned
            .into_iter()
            .map(|id| WorldChange::Spawn(id, filter.read_entity_components(to, id).into()));
        let despawned = despawned.into_iter().map(WorldChange::Despawn);
        let updated = in_both.into_iter().flat_map(|id| {
            let from_comps: HashMap<_, _> = filter
                .get_entity_components(from, id)
                .into_iter()
                .map(|v| (v.index(), v))
                .collect();
            let to_comps: HashMap<_, _> = filter
                .get_entity_components(to, id)
                .into_iter()
                .map(|v| (v.index(), v))
                .collect();

            let added = to_comps
                .iter()
                .filter(|c| !from_comps.contains_key(c.0))
                .map(|v| *v.1)
                .collect_vec();
            let removed = from_comps
                .iter()
                .filter(|c| !to_comps.contains_key(c.0))
                .map(|v| *v.1)
                .collect_vec();
            let in_both = to_comps
                .iter()
                .filter(|c| from_comps.contains_key(c.0))
                .collect_vec();

            let changed = in_both
                .iter()
                .filter_map(|&c| {
                    (from.get_component_content_version(id, *c.0).unwrap()
                        != to.get_component_content_version(id, *c.0).unwrap())
                    .then_some(c.1)
                })
                .collect_vec();

            let added: Entity = added
                .iter()
                .map(|&comp| to.get_entry(id, comp).unwrap())
                .collect();

            let added = if !added.is_empty() {
                vec![WorldChange::AddComponents(id, added)]
            } else {
                vec![]
            };
            let removed = if !removed.is_empty() {
                vec![WorldChange::RemoveComponents(id, removed)]
            } else {
                vec![]
            };
            let changed: Entity = changed
                .into_iter()
                .map(|&comp| to.get_entry(id, comp).unwrap())
                .collect();
            added
                .into_iter()
                .chain(removed)
                .chain(std::iter::once(WorldChange::SetComponents(id, changed)))
        });

        Self {
            changes: despawned.chain(spawned).chain(updated).collect_vec(),
        }
    }

    pub fn len(&self) -> usize {
        self.changes.len()
    }
}
impl Display for WorldDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for change in self.changes.iter().take(3) {
            change.fmt(f)?;
            write!(f, " ").unwrap();
        }
        if self.changes.len() > 3 {
            write!(f, "...{} more", self.changes.len() - 3).unwrap();
        }
        Ok(())
    }
}
impl<'a> IntoIterator for &'a WorldDiff {
    type Item = &'a WorldChange;

    type IntoIter = core::slice::Iter<'a, WorldChange>;

    fn into_iter(self) -> Self::IntoIter {
        self.changes.iter()
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct WorldDiffView<'a> {
    pub changes: Vec<Cow<'a, WorldChange>>,
}

impl<'a> From<&'a WorldDiff> for WorldDiffView<'a> {
    fn from(value: &'a WorldDiff) -> Self {
        WorldDiffView {
            changes: value.changes.iter().map(Cow::Borrowed).collect(),
        }
    }
}

impl<'a> IntoIterator for &'a WorldDiffView<'a> {
    type Item = &'a WorldChange;

    type IntoIter = WorldDiffViewIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        WorldDiffViewIter {
            inner: self.changes.iter(),
        }
    }
}
pub struct WorldDiffViewIter<'a> {
    inner: core::slice::Iter<'a, Cow<'a, WorldChange>>,
}
impl<'a> Iterator for WorldDiffViewIter<'a> {
    type Item = &'a WorldChange;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(AsRef::as_ref)
    }
}
impl<'a> ExactSizeIterator for WorldDiffViewIter<'a> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

/// Immutable version of WorldDiff, cheap to clone
#[derive(Serialize, Clone, Debug)]
pub struct FrozenWorldDiff {
    changes: Arc<[WorldChange]>,
}
impl FrozenWorldDiff {
    pub fn merge(diffs: &[Self]) -> WorldDiffView<'_> {
        // indexes of the last SetComponents for each entity
        let mut set_idx: HashMap<EntityId, usize> = HashMap::new();
        // merged changes
        let mut changes = Vec::new();

        for change in diffs.iter().flat_map(|diff| diff.changes.iter()) {
            if let WorldChange::SetComponents(entity_id, entity) = change {
                if let Some(idx) = set_idx.get(entity_id) {
                    if let Some(WorldChange::SetComponents(_, existing_entity)) =
                        changes.get_mut(*idx).map(Cow::to_mut)
                    {
                        existing_entity.merge(entity.clone());
                    } else {
                        // all indexes in set_idx should point to SetComponents in changes vec
                        unreachable!();
                    }
                } else {
                    set_idx.insert(*entity_id, changes.len());
                    changes.push(Cow::Borrowed(change));
                }
            } else {
                changes.push(Cow::Borrowed(change));
            }
        }

        tracing::debug!(
            "Merged {} changes into {}",
            diffs.iter().map(|diff| diff.changes.len()).sum::<usize>(),
            changes.len(),
        );

        WorldDiffView { changes }
    }
}
impl From<WorldDiff> for FrozenWorldDiff {
    fn from(diff: WorldDiff) -> Self {
        Self {
            changes: diff.changes.into(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WorldStreamCompEvent {
    Init,
    Set,
    Spawn,
    AddComponent,
    RemoveComponent,
}

#[derive(Clone)]
pub struct WorldStreamFilter {
    arch_filter: ArchetypeFilter,
    component_filter: Arc<dyn Fn(ComponentDesc, WorldStreamCompEvent) -> bool + Sync + Send>,
}
impl WorldStreamFilter {
    pub fn new(
        arch_filter: ArchetypeFilter,
        component_filter: Arc<dyn Fn(ComponentDesc, WorldStreamCompEvent) -> bool + Sync + Send>,
    ) -> Self {
        Self {
            arch_filter,
            component_filter,
        }
    }
    pub fn initial_diff(&self, world: &World) -> WorldDiff {
        WorldDiff {
            changes: self
                .all_entities(world)
                .map(|id| WorldChange::Spawn(id, self.read_entity_components(world, id).into()))
                .collect_vec(),
        }
    }
    pub fn all_entities<'a>(&self, world: &'a World) -> impl Iterator<Item = EntityId> + 'a {
        Query::all()
            .filter(&self.arch_filter)
            .iter(world, None)
            .map(|x| x.id())
    }
    pub fn get_entity_components(&self, world: &World, id: EntityId) -> Vec<ComponentDesc> {
        world
            .get_components(id)
            .unwrap()
            .into_iter()
            .filter(|&comp| (self.component_filter)(comp, WorldStreamCompEvent::Init))
            .collect_vec()
    }
    fn read_entity_components(&self, world: &World, id: EntityId) -> Vec<ComponentEntry> {
        self.get_entity_components(world, id)
            .into_iter()
            .map(|comp| world.get_entry(id, comp).unwrap())
            .collect_vec()
    }
}
impl Default for WorldStreamFilter {
    fn default() -> Self {
        Self {
            arch_filter: Default::default(),
            component_filter: Arc::new(|_, _| true),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum WorldChange {
    Spawn(EntityId, Entity),
    Despawn(EntityId),
    AddComponents(EntityId, Entity),
    RemoveComponents(EntityId, Vec<ComponentDesc>),
    SetComponents(EntityId, Entity),
}

impl WorldChange {
    pub fn is_set(&self) -> bool {
        matches!(self, Self::SetComponents(_, _))
    }

    pub fn is_remove_components(&self) -> bool {
        matches!(self, Self::RemoveComponents(_, _))
    }

    fn apply(self, world: &mut World, spawned_extra_data: &Entity, panic_on_error: bool) {
        match self {
            Self::Spawn(id, data) => {
                if !world.spawn_with_id(id, data.with_merge(spawned_extra_data.clone())) {
                    if panic_on_error {
                        panic!("WorldChange::apply spawn_mirror entity already exists: {id:?}");
                    } else {
                        log::error!(
                            "WorldChange::apply spawn_mirror entity already exists: {id:?}"
                        );
                    }
                }
            }
            Self::Despawn(id) => {
                world.despawn(id);
            }
            Self::AddComponents(id, data) => {
                world.add_components(id, data).unwrap();
            }
            Self::RemoveComponents(id, comps) => {
                world.remove_components(id, comps).unwrap();
            }
            Self::SetComponents(id, data) => {
                if let Err(err) = world.set_components(id, data) {
                    if panic_on_error {
                        panic!("Failed to set: {err:?}");
                    }
                };
            }
        }
    }
    fn filter(&self, world: &World, filter: &WorldStreamFilter) -> Option<Self> {
        match self {
            Self::Spawn(id, data) => {
                if !filter.arch_filter.matches_entity(world, *id) {
                    return None;
                }
                let mut data = data.clone();
                data.filter(&|comp| (filter.component_filter)(comp, WorldStreamCompEvent::Spawn));
                Some(Self::Spawn(*id, data.clone()))
            }
            Self::Despawn(id) => {
                // TODO: Right now we don't filter despawns, because the spawns are filtered, so the "bad" despawns will just be
                // ignored on the client side. Maybe should filter them on the server side too
                Some(Self::Despawn(*id))
            }
            Self::AddComponents(id, data) => {
                if !filter.arch_filter.matches_entity(world, *id) {
                    return None;
                }
                let mut data = data.clone();
                data.filter(&|comp| {
                    (filter.component_filter)(comp, WorldStreamCompEvent::AddComponent)
                });
                if data.is_empty() {
                    return None;
                }
                Some(Self::AddComponents(*id, data.clone()))
            }
            Self::RemoveComponents(id, comps) => {
                if !filter.arch_filter.matches_entity(world, *id) {
                    return None;
                }
                Some(Self::RemoveComponents(
                    *id,
                    comps
                        .iter()
                        .filter_map(|&comp| {
                            if (filter.component_filter)(
                                comp,
                                WorldStreamCompEvent::RemoveComponent,
                            ) {
                                Some(comp)
                            } else {
                                None
                            }
                        })
                        .collect_vec(),
                ))
            }
            Self::SetComponents(_, _) => Some(self.clone()),
        }
    }
}
impl Display for WorldChange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorldChange::Spawn(id, data) => write!(f, "spawn({}, {})", id, data.len()),
            WorldChange::Despawn(id) => write!(f, "despawn({id})"),
            WorldChange::AddComponents(id, data) => write!(f, "add_components({id}, {data:?})"),
            WorldChange::RemoveComponents(id, _) => write!(f, "remove_components({id})"),
            WorldChange::SetComponents(id, data) => write!(f, "set({id}, {data:?})"),
        }
    }
}

#[derive(Clone)]
pub struct WorldStream {
    changed_qs: QueryState,
    shape_stream_reader: FramedEventsReader<WorldChange>,
    filter: WorldStreamFilter,
    version: u64,
}
impl WorldStream {
    pub fn new(filter: WorldStreamFilter) -> Self {
        Self {
            changed_qs: QueryState::new(),
            shape_stream_reader: FramedEventsReader::new(),
            filter,
            version: 0,
        }
    }
    pub fn filter(&self) -> &WorldStreamFilter {
        &self.filter
    }
    #[profiling::function]
    pub fn next_diff(&mut self, world: &World) -> WorldDiff {
        // get all shape changes (spawn/despawn/add/remove components)
        let shape_changes = self
            .shape_stream_reader
            .iter(world.shape_change_events.as_ref().unwrap())
            .filter_map(|(_, change)| change.filter(world, &self.filter))
            .collect_vec();

        // prepare a list of entities/components that were removed so we don't create Set operations for them
        let mut removed_entities = HashSet::new();
        let mut removed_components = HashSet::new();
        for change in shape_changes.iter() {
            match change {
                WorldChange::Spawn(id, _) => {
                    removed_entities.remove(id);
                }
                WorldChange::Despawn(id) => {
                    removed_entities.insert(*id);
                }
                WorldChange::AddComponents(id, comps) => {
                    for entry in comps.iter() {
                        removed_components.remove(&(*id, entry.desc()));
                    }
                }
                WorldChange::RemoveComponents(id, comps) => {
                    removed_components.extend(comps.iter().map(|&desc| (*id, desc)));
                }
                _ => {}
            }
        }

        // get all Set operations
        let mut sets = HashMap::new();
        for arch in world.archetypes.iter() {
            if self.filter.arch_filter.matches(&arch.active_components) {
                for arch_comp in arch.components.iter() {
                    if (self.filter.component_filter)(
                        arch_comp.component,
                        WorldStreamCompEvent::Set,
                    ) {
                        let reader = self
                            .changed_qs
                            .change_readers
                            .get(arch.id, arch_comp.component.index() as _);

                        for (_, &entity_id) in reader.iter(&*arch_comp.changes.borrow()) {
                            if removed_entities.contains(&entity_id) {
                                // don't create Set operation if the entity was removed
                                continue;
                            }
                            if let Some(loc) = world.entity_loc(entity_id) {
                                if loc.archetype == arch.id
                                    && arch_comp.get_content_version(loc.index) > self.version
                                    && !removed_components
                                        .contains(&(entity_id, arch_comp.component))
                                {
                                    let entry = sets.entry(entity_id).or_insert_with(Entity::new);
                                    entry.set_entry(
                                        world.get_entry(entity_id, arch_comp.component).unwrap(),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        self.version = world.version();
        let mut changes = shape_changes;
        changes.extend(
            sets.into_iter()
                .map(|(id, entity)| WorldChange::SetComponents(id, entity)),
        );
        WorldDiff { changes }
    }
}
