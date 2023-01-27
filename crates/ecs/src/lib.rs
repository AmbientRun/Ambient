#![allow(incomplete_features)]
#![feature(type_alias_impl_trait)]

use core::fmt;
use std::{
    collections::{HashMap, HashSet}, fmt::{Debug, Formatter}, fs::File, iter::once, path::Path, sync::atomic::{AtomicU64, Ordering}
};

use anyhow::Context;
use bit_set::BitSet;
use bit_vec::BitVec;
use elements_std::sparse_vec::SparseVec;
use itertools::Itertools;
/// Expose to macros
#[doc(hidden)]
pub use once_cell::sync::OnceCell;
/// Expose to macros
#[doc(hidden)]
pub use parking_lot;
use parking_lot::Mutex;
/// Expose to macros
#[doc(hidden)]
pub use paste;
use serde::{Deserialize, Serialize};
use thiserror::Error;

mod archetype;
mod attributes;
mod component;
pub mod component2;
mod component_entry;
mod component_registry;
mod component_unit;
mod entity_data;
mod entity_uid;
mod events;
mod index;
mod location;
mod primitive_component;
mod query;
mod serialization;
mod stream;
pub use archetype::*;
pub use attributes::*;
pub use component2::{Component, ComponentDesc, ComponentValue, ComponentValueBase};
pub use component_entry::*;
pub use component_registry::*;
pub use component_unit::*;
pub use entity_data::*;
pub use entity_uid::*;
pub use events::*;
pub use index::*;
pub use location::*;
pub use primitive_component::*;
pub use query::*;
pub use serialization::*;
pub use stream::*;

pub struct DebugWorldArchetypes<'a> {
    world: &'a World,
}

impl<'a> Debug for DebugWorldArchetypes<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_map();

        for arch in &self.world.archetypes {
            s.entry(&arch.id, &arch.dump_info());
        }

        s.finish()
    }
}

components!("ecs", {
    @[Networked, Store, Debuggable]
    id: EntityId,
    @[Networked, Store, Debuggable]
    ids: Vec<EntityId>,
    /// Generic component that indicates the entity shouldn't be stored on disk
    @[Networked, Store, Debuggable]
    dont_store: (),

    /// User-editable entities need to have a `uid` component, so that intent history and selection can
    /// work, even if the actual entity in the world is destroyed and then re-created
    @[Networked, Store, Debuggable]
    uid: EntityUid,
    @[Debuggable]
    uid_lookup: UidLookup,
});

#[derive(Clone)]
pub struct World {
    name: &'static str,
    archetypes: Vec<Archetype>,
    locs: EntityLocations,
    loc_changed: FramedEvents<EntityId>,
    version: CloneableAtomicU64,
    shape_change_events: Option<FramedEvents<WorldChange>>,
    /// Used for reset_events. Prevents change events in queries when you use reset_events
    ignore_query_inits: bool,
}
impl World {
    pub fn new(name: &'static str) -> Self {
        Self::new_with_config(name, 0, true)
    }
    pub fn new_with_config(name: &'static str, namespace: u8, resources: bool) -> Self {
        Self::new_with_config_internal(name, namespace, if resources { CreateResources::Create } else { CreateResources::AllocateLoc })
    }
    fn new_with_config_internal(name: &'static str, namespace: u8, resources: CreateResources) -> Self {
        let mut world = Self {
            name,
            archetypes: Vec::new(),
            locs: EntityLocations::new(0),
            loc_changed: FramedEvents::new(),
            version: CloneableAtomicU64::new(0),
            shape_change_events: None,
            ignore_query_inits: false,
        };
        match resources {
            CreateResources::Create => {
                world.spawn(EntityData::new());
            }
            CreateResources::AllocateLoc => {
                // Reserve 0:0:0 for resources
                let id = world.spawn(EntityData::new());
                world.despawn(id);
            }
            CreateResources::None => {}
        }
        if namespace != 0 {
            world.set_namespace(namespace);
        }
        world
    }
    /// Clones all entities specified in the source world and returns a new world with them
    pub fn from_entities(world: &World, entities: impl IntoIterator<Item = EntityId>, serialiable_only: bool) -> Self {
        let mut res = World::new_with_config("from_entities", 0, false);
        for id in entities {
            let mut entity = world.clone_entity(id).unwrap();
            if serialiable_only {
                for comp in entity.components() {
                    if comp.attribute::<Serializable>().is_none() {
                        entity.remove_raw(comp);
                    }
                }
            }
            entity.spawn(&mut res);
        }
        res
    }
    pub async fn from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let content = tokio::fs::read(&path).await.with_context(|| format!("No such file: {:?}", path.as_ref()))?;
        Self::from_slice(&content)
    }
    pub fn from_slice(content: &[u8]) -> anyhow::Result<Self> {
        let DeserWorldWithWarnings { world, warnings } = serde_json::from_slice(content)?;
        warnings.log_warnings();
        Ok(world)
    }
    pub fn spawn(&mut self, entity_data: EntityData) -> EntityId {
        self.batch_spawn(entity_data, 1).pop().unwrap()
    }
    pub fn batch_spawn(&mut self, entity_data: EntityData, count: usize) -> Vec<EntityId> {
        let ids = self.locs.allocate(count);
        if let Some(events) = &mut self.shape_change_events {
            events.add_events(ids.iter().map(|id| WorldChange::Spawn(Some(*id), entity_data.clone())));
        }
        self.spawn_with_ids(EntityMoveData::from_entity_data(entity_data, self.version() + 1), ids.clone());
        ids
    }
    /// Spawn an entity which lives remotely, and is just mirrored locally; i.e. the id is managed
    /// on the remote side. Returns false if the id already exists
    pub fn spawn_mirrored(&mut self, entity_id: EntityId, entity_data: EntityData) -> bool {
        if self.locs.allocate_mirror(entity_id) {
            self.spawn_with_ids(EntityMoveData::from_entity_data(entity_data, self.version() + 1), vec![entity_id]);
            true
        } else {
            false
        }
    }
    fn spawn_with_ids(&mut self, entity_data: EntityMoveData, ids: Vec<EntityId>) {
        self.inc_version();
        let arch_id = self.archetypes.iter().position(|x| x.active_components == entity_data.active_components);
        let arch_id = if let Some(arch_id) = arch_id {
            arch_id
        } else {
            let arch_id = self.archetypes.len();
            self.archetypes.push(Archetype::new(arch_id, entity_data.components()));
            arch_id
        };
        let arch = &mut self.archetypes[arch_id];
        for (i, id) in ids.iter().enumerate() {
            let loc = self.locs.get_mut(*id).expect("No such entity id");
            loc.archetype = arch.id;
            loc.index = arch.next_index() + i;
        }
        arch.movein(ids, entity_data);
    }
    pub fn despawn(&mut self, entity_id: EntityId) -> Option<EntityData> {
        if let Some(loc) = self.locs.get(entity_id) {
            let version = self.inc_version();
            if let Some(events) = &mut self.shape_change_events {
                events.add_event(WorldChange::Despawn(entity_id));
            }
            let arch = self.archetypes.get_mut(loc.archetype).expect("No such archetype");
            let last_entity_in_arch = *arch.entity_indices_to_ids.last().unwrap();
            self.locs.free(loc, entity_id, last_entity_in_arch);
            if last_entity_in_arch != entity_id {
                self.loc_changed.add_event(last_entity_in_arch);
            }
            Some(arch.moveout(loc.index, entity_id, version).into())
        } else {
            None
        }
    }
    pub fn despawn_all(&mut self) {
        let entity_ids: Vec<EntityId> = query_mut((), ()).iter(self, None).map(|(id, _, _)| id).collect();
        for id in entity_ids {
            self.despawn(id);
        }
    }
    #[profiling::function]
    pub fn next_frame(&mut self) {
        for arch in &mut self.archetypes {
            arch.next_frame();
        }
        if let Some(events) = &mut self.shape_change_events {
            events.next_frame();
        }
        self.ignore_query_inits = false;
    }

    pub fn set<T: ComponentValue>(
        &mut self,
        entity_id: EntityId,
        component: crate::component2::Component<T>,
        value: T,
    ) -> Result<T, ECSError> {
        let p = self.get_mut(entity_id, component)?;
        Ok(std::mem::replace(p, value))
    }

    pub fn set_entry(&mut self, entity_id: EntityId, entry: ComponentEntry) -> Result<ComponentEntry, ECSError> {
        if let Some(loc) = self.locs.get(entity_id) {
            let version = self.inc_version();
            let arch = self.archetypes.get_mut(loc.archetype).expect("Archetype doesn't exist");
            arch.replace_with_entry(entity_id, loc.index, entry, version)
        } else {
            Err(ECSError::NoSuchEntity { entity_id })
        }
    }

    /// Sets the value iff it is different to the current
    pub fn set_if_changed<T: ComponentValue + PartialEq>(
        &mut self,
        entity_id: EntityId,
        component: Component<T>,
        value: T,
    ) -> Result<(), ECSError> {
        let old = self.get_ref(entity_id, component)?;
        if old != &value {
            self.set(entity_id, component, value)?;
        }
        Ok(())
    }
    pub fn get_mut<T: ComponentValue>(&mut self, entity_id: EntityId, component: Component<T>) -> Result<&mut T, ECSError> {
        self.get_mut_unsafe(entity_id, component)
    }
    pub(crate) fn get_mut_unsafe<T: ComponentValue>(&self, entity_id: EntityId, component: Component<T>) -> Result<&mut T, ECSError> {
        if let Some(loc) = self.locs.get(entity_id) {
            self.inc_version();
            let arch = self.archetypes.get(loc.archetype).expect("Archetype doesn't exist");
            match arch.get_component_mut(loc.index, entity_id, component, self.version()) {
                Some(d) => Ok(d),
                None => Err(ECSError::EntityDoesntHaveComponent { component_index: component.desc().index() as _, name: component.name() }),
            }
        } else {
            Err(ECSError::NoSuchEntity { entity_id })
        }
    }
    pub fn get<T: Copy + ComponentValue>(&self, entity_id: EntityId, component: Component<T>) -> Result<T, ECSError> {
        self.get_ref(entity_id, component).map(|x| *x)
    }
    pub fn get_cloned<T: Clone + ComponentValue>(&self, entity_id: EntityId, component: Component<T>) -> Result<T, ECSError> {
        self.get_ref(entity_id, component).map(|x| x.clone())
    }
    pub fn get_ref<T: ComponentValue>(&self, entity_id: EntityId, component: Component<T>) -> Result<&T, ECSError> {
        if let Some(loc) = self.locs.get(entity_id) {
            let arch = self.archetypes.get(loc.archetype).expect("Archetype doesn't exist");
            match arch.get_component(loc.index, component) {
                Some(d) => Ok(d),
                None => Err(ECSError::EntityDoesntHaveComponent {
                    component_index: component.desc().index() as usize,
                    name: component.name().into(),
                }),
            }
        } else {
            Err(ECSError::NoSuchEntity { entity_id })
        }
    }
    pub fn get_entry(&self, entity_id: EntityId, component: ComponentDesc) -> Result<ComponentEntry, ECSError> {
        if let Some(loc) = self.locs.get(entity_id) {
            let arch = self.archetypes.get(loc.archetype).expect("Archetype doesn't exist");
            match arch.get_component_buffer_untyped(component) {
                Some(d) => Ok(d.clone_value_boxed(loc.index)),
                None => {
                    Err(ECSError::EntityDoesntHaveComponent { component_index: component.index() as usize, name: component.name().into() })
                }
            }
        } else {
            Err(ECSError::NoSuchEntity { entity_id })
        }
    }
    pub fn has_component_index(&self, entity_id: EntityId, component_index: u32) -> bool {
        if let Some(loc) = self.locs.get(entity_id) {
            let arch = self.archetypes.get(loc.archetype).expect("Archetype doesn't exist");
            arch.active_components.contains_index(component_index as usize)
        } else {
            false
        }
    }
    #[inline]
    pub fn has_component_ref(&self, entity_id: EntityId, component: impl Into<ComponentDesc>) -> bool {
        self.has_component_index(entity_id, component.into().index() as _)
    }
    #[inline]
    pub fn has_component(&self, entity_id: EntityId, component: impl Into<ComponentDesc>) -> bool {
        self.has_component_ref(entity_id, component.into())
    }
    pub fn get_components(&self, entity_id: EntityId) -> Result<Vec<ComponentDesc>, ECSError> {
        if let Some(loc) = self.locs.get(entity_id) {
            let arch = self.archetypes.get(loc.archetype).expect("Archetype doesn't exist");
            Ok(arch.components.iter().map(|x| x.component).collect_vec())
        } else {
            Err(ECSError::NoSuchEntity { entity_id })
        }
    }

    pub fn clone_entity(&self, entity_id: EntityId) -> Result<EntityData, ECSError> {
        self.get_components(entity_id).map(|components| {
            let mut ed = EntityData::new();
            for comp in components {
                ed.set_entry(self.get_entry(entity_id, comp).unwrap());
            }
            ed
        })
    }

    pub fn entities(&self) -> Vec<(EntityId, EntityData)> {
        query(()).iter(self, None).map(|(id, _)| (id, self.clone_entity(id).unwrap())).collect()
    }
    pub fn exists(&self, entity_id: EntityId) -> bool {
        self.locs.exists(entity_id)
    }

    fn map_entity(&mut self, entity_id: EntityId, map: impl FnOnce(MapEntity) -> MapEntity) -> Result<(), ECSError> {
        if let Some(loc) = self.locs.get(entity_id) {
            let version = self.inc_version();
            let prev_comps = self.archetypes.get_mut(loc.archetype).expect("No such archetype").active_components.clone();

            let mapping = map(MapEntity { sets: HashMap::new(), removes: HashSet::new(), active_components: prev_comps.clone() });

            if mapping.active_components == prev_comps {
                assert_eq!(mapping.removes.len(), 0);
                let arch = self.archetypes.get_mut(loc.archetype).expect("No such archetype");
                for (desc, value) in mapping.sets.into_iter() {
                    arch.set_component_raw(loc.index, entity_id, value, version);
                }
            } else {
                let arch = self.archetypes.get_mut(loc.archetype).expect("No such archetype");
                let last_entity_in_arch = *arch.entity_indices_to_ids.last().unwrap();
                self.locs.on_swap_remove(loc, last_entity_in_arch);
                self.loc_changed.add_event(last_entity_in_arch);
                self.loc_changed.add_event(entity_id);
                let mut data = arch.moveout(loc.index, entity_id, version);
                mapping.write_to_entity_data(&mut data, version);
                self.spawn_with_ids(data, vec![entity_id]);
            }
            Ok(())
        } else {
            Err(ECSError::NoSuchEntity { entity_id })
        }
    }

    pub fn add_components(&mut self, entity_id: EntityId, data: EntityData) -> Result<(), ECSError> {
        if let Some(events) = &mut self.shape_change_events {
            events.add_event(WorldChange::AddComponents(entity_id, data.clone()));
        }
        self.map_entity(entity_id, |ed| ed.append(data))
    }
    // will also replace the existing component of the same type if it exists
    pub fn add_component<T: ComponentValue>(&mut self, entity_id: EntityId, component: Component<T>, value: T) -> Result<(), ECSError> {
        self.add_components(entity_id, EntityData::new().set(component, value))
    }

    pub fn add_resource<T: ComponentValue>(&mut self, component: Component<T>, value: T) {
        self.add_component(self.resource_entity(), component, value).unwrap()
    }

    /// Does nothing if the component does not exist
    pub fn remove_component(&mut self, entity_id: EntityId, component: impl Into<ComponentDesc>) -> Result<(), ECSError> {
        self.remove_components(entity_id, vec![component.into()])
    }

    pub fn remove_components(&mut self, entity_id: EntityId, components: Vec<ComponentDesc>) -> Result<(), ECSError> {
        if let Some(events) = &mut self.shape_change_events {
            events.add_event(WorldChange::RemoveComponents(entity_id, components.clone()));
        }
        self.map_entity(entity_id, |entity| entity.remove_components(components))
    }
    pub fn resource_entity(&self) -> EntityId {
        EntityId { namespace: 0, id: 0, gen: 0 }
    }
    pub fn resource_opt<T: ComponentValue>(&self, component: Component<T>) -> Option<&T> {
        self.get_ref(self.resource_entity(), component).ok()
    }
    pub fn resource<T: ComponentValue>(&self, component: Component<T>) -> &T {
        match self.resource_opt(component) {
            Some(val) => val,
            None => panic!("Resource {} does not exist", component.name()),
        }
    }
    pub fn resource_mut_opt<T: ComponentValue>(&mut self, component: Component<T>) -> Option<&mut T> {
        self.get_mut(self.resource_entity(), component).ok()
    }
    pub fn resource_mut<T: ComponentValue>(&mut self, component: Component<T>) -> &mut T {
        self.resource_mut_opt(component).unwrap()
    }
    pub fn archetypes(&self) -> &Vec<Archetype> {
        &self.archetypes
    }
    pub fn entity_loc(&self, id: EntityId) -> Option<EntityLocation> {
        self.locs.get(id)
    }
    /// Returns the content version of this component, which only changes when the component is written to (not when the entity changes archetype)
    pub fn get_component_content_version(&self, entity_id: EntityId, index: u32) -> Result<u64, ECSError> {
        if let Some(loc) = self.locs.get(entity_id) {
            let arch = self.archetypes.get(loc.archetype).expect("Archetype doesn't exist");
            match arch.get_component_content_version(loc, index) {
                Some(d) => Ok(d),
                None => Err(ECSError::EntityDoesntHaveComponent { component_index: index as _, name: "".to_string() }),
            }
        } else {
            Err(ECSError::NoSuchEntity { entity_id })
        }
    }
    pub fn loc_changed(&self) -> &FramedEvents<EntityId> {
        &self.loc_changed
    }
    pub fn init_shape_change_tracking(&mut self) {
        self.shape_change_events = Some(FramedEvents::new());
    }
    pub fn reset_events(&mut self) {
        self.loc_changed = FramedEvents::new();
        if let Some(shape_change_events) = &mut self.shape_change_events {
            *shape_change_events = FramedEvents::new();
        }
        for arch in self.archetypes.iter_mut() {
            arch.reset_events();
        }
        self.ignore_query_inits = true;
    }
    /// Spawn all entities of this world into the destination world
    pub fn spawn_into_world(&self, world: &mut World, components: Option<EntityData>) -> Vec<EntityId> {
        let mut old_to_new_ids = HashMap::new();
        for (old_id, mut entity) in self.entities().into_iter() {
            if old_id != self.resource_entity() {
                if let Some(components) = components.as_ref() {
                    entity.append_self(components.clone());
                }
                let new_id = entity.spawn(world);
                old_to_new_ids.insert(old_id, new_id);
            }
        }

        let migraters = COMPONENT_ENTITY_ID_MIGRATERS.lock();
        for migrater in migraters.iter() {
            for id in old_to_new_ids.values() {
                migrater(world, *id, &old_to_new_ids);
            }
        }
        old_to_new_ids.into_values().collect()
    }
    fn version(&self) -> u64 {
        self.version.0.load(Ordering::Relaxed)
    }
    fn inc_version(&self) -> u64 {
        self.version.0.fetch_add(1, Ordering::Relaxed)
    }
    /// Number of entities in the world, including the resource entity
    pub fn len(&self) -> usize {
        self.archetypes.iter().fold(0, |p, x| p + x.entity_count())
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn set_namespace(&mut self, namespace: u8) {
        self.locs.set_namespace(namespace);
    }

    pub fn debug_archetypes(&self) -> DebugWorldArchetypes {
        DebugWorldArchetypes { world: self }
    }

    pub fn dump(&self, f: &mut dyn std::io::Write) {
        for arch in &self.archetypes {
            if arch.entity_count() > 0 {
                arch.dump(f);
            }
        }
    }
    pub fn dump_to_tmp_file(&self) {
        std::fs::create_dir_all("tmp").ok();
        let mut f = File::create("tmp/ecs.txt").expect("Unable to create file");
        self.dump(&mut f);
        println!("Wrote ecs to tmp/ecs.txt");
    }
    pub fn dump_entity(&self, entity_id: EntityId, indent: usize, f: &mut dyn std::io::Write) {
        if let Some(loc) = self.locs.get(entity_id) {
            let arch = self.archetypes.get(loc.archetype).expect("No such archetype");

            arch.dump_entity(loc.index, indent, f);
        } else {
            let indent = format!("{:indent$}", "", indent = indent);
            writeln!(f, "{indent}ERROR, NO SUCH ENTITY: {}", entity_id).unwrap();
        }
    }

    pub fn dump_entity_to_yml(&self, entity_id: EntityId) -> Option<(String, yaml_rust::yaml::Hash)> {
        if let Some(loc) = self.locs.get(entity_id) {
            let arch = self.archetypes.get(loc.archetype).expect("No such archetype");
            Some(arch.dump_entity_to_yml(loc.index))
        } else {
            None
        }
    }

    pub fn set_name(&mut self, name: &'static str) {
        self.name = name;
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn add_entry(&mut self, id: EntityId, entry: ComponentEntry) -> Result<(), ECSError> {
        self.add_components(id, once(entry).collect())
    }
}

impl std::fmt::Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("World").finish()
    }
}

unsafe impl Send for World {}
unsafe impl Sync for World {}

enum CreateResources {
    Create,
    AllocateLoc,
    None,
}

// TODO(fred): Move this into the actual components instead
pub static COMPONENT_ENTITY_ID_MIGRATERS: Mutex<Vec<fn(&mut World, EntityId, &HashMap<EntityId, EntityId>)>> = Mutex::new(Vec::new());

#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub enum ECSError {
    #[error("Entity doesn't have component: {component_index} {name}")]
    EntityDoesntHaveComponent { component_index: usize, name: String },
    #[error("No such entity: {entity_id}")]
    NoSuchEntity { entity_id: EntityId },
}

struct MapEntity {
    sets: HashMap<u32, ComponentEntry>,
    removes: HashSet<u32>,
    active_components: ComponentSet,
}
impl MapEntity {
    fn append(mut self, other: EntityData) -> Self {
        for entry in other {
            self.active_components.insert(entry.desc());
            self.sets.insert(entry.desc().index() as _, entry);
        }
        self
    }

    fn remove_components(mut self, components: Vec<ComponentDesc>) -> Self {
        for desc in components {
            if self.active_components.contains(desc) {
                self.active_components.remove(desc);
                self.removes.insert(desc.index() as _);
            }
        }
        self
    }
    fn write_to_entity_data(self, data: &mut EntityMoveData, version: u64) {
        for value in self.sets.into_values() {
            data.set(value, version);
        }

        for comp in self.removes.into_iter() {
            data.remove(comp as _);
        }
    }
}

pub enum Command {
    Set(EntityId, ComponentEntry),
    AddComponent(EntityId, ComponentEntry),
    RemoveComponent(EntityId, ComponentDesc),
    Despawn(EntityId),
    Defer(Box<dyn Fn(&mut World) -> Result<(), ECSError> + Sync + Send + 'static>),
}

impl Command {
    fn apply(self, world: &mut World) -> Result<(), ECSError> {
        match self {
            Command::Set(id, entry) => {
                world.set_entry(id, entry)?;
                Ok(())
            }
            Command::AddComponent(entity, entry) => world.add_entry(entity, entry),
            Command::RemoveComponent(entity, component) => world.remove_component(entity, component),
            Command::Despawn(id) => {
                if world.despawn(id).is_none() {
                    Err(ECSError::NoSuchEntity { entity_id: id })
                } else {
                    Ok(())
                }
            }
            Command::Defer(func) => func(world),
        }
    }
}
pub struct Commands(Vec<Command>);
impl Commands {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn set<T: ComponentValue>(&mut self, entity_id: EntityId, component: Component<T>, value: impl Into<T>) {
        self.0.push(Command::Set(entity_id, ComponentEntry::new(component, value.into())))
    }
    pub fn add_component<T: ComponentValue>(&mut self, entity_id: EntityId, component: Component<T>, value: T) {
        self.0.push(Command::AddComponent(entity_id, ComponentEntry::new(component, value)))
    }
    pub fn remove_component<T: ComponentValue>(&mut self, entity_id: EntityId, component: impl Into<ComponentDesc>) {
        self.0.push(Command::RemoveComponent(entity_id, component.into()));
    }
    pub fn despawn(&mut self, entity_id: EntityId) {
        self.0.push(Command::Despawn(entity_id));
    }

    /// Defers a function to execute upon the world.
    pub fn defer(&mut self, func: impl Fn(&mut World) -> Result<(), ECSError> + Sync + Send + 'static) {
        self.0.push(Command::Defer(Box::new(func)))
    }

    pub fn apply(&mut self, world: &mut World) -> Result<(), ECSError> {
        for command in self.0.drain(..) {
            command.apply(world)?;
        }
        Ok(())
    }
    /// Like apply, but doesn't stop on an error, instead just logs a warning
    pub fn soft_apply(&mut self, world: &mut World) {
        for command in self.0.drain(..) {
            if let Err(err) = command.apply(world) {
                log::warn!("soft_apply error: {:?}", err);
            }
        }
    }
    /// Like soft apply, but doesn't even issue a warning
    pub fn softer_apply(&mut self, world: &mut World) {
        for command in self.0.drain(..) {
            command.apply(world).ok();
        }
    }
}

pub(crate) struct CloneableAtomicU64(pub AtomicU64);
impl CloneableAtomicU64 {
    pub fn new(value: u64) -> Self {
        Self(AtomicU64::new(value))
    }
}
impl Clone for CloneableAtomicU64 {
    fn clone(&self) -> Self {
        Self(AtomicU64::new(self.0.load(Ordering::SeqCst)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentSet(pub BitSet);
impl ComponentSet {
    pub fn new() -> Self {
        Self(BitSet::with_capacity(with_component_registry(|cr| cr.component_count())))
    }

    pub fn insert(&mut self, component: ComponentDesc) {
        self.0.insert(component.index() as _);
    }
    pub fn remove(&mut self, component: ComponentDesc) {
        self.remove_by_index(component.index() as _)
    }
    pub fn remove_by_index(&mut self, component_index: usize) {
        self.0.remove(component_index);
    }
    pub fn union_with(&mut self, rhs: &ComponentSet) {
        self.0.union_with(&rhs.0);
    }

    #[inline]
    pub fn contains(&self, desc: ComponentDesc) -> bool {
        self.contains_index(desc.index() as _)
    }
    pub fn contains_index(&self, component_index: usize) -> bool {
        self.0.contains(component_index)
    }
    pub fn is_superset(&self, other: &ComponentSet) -> bool {
        self.0.is_superset(&other.0)
    }
    pub fn is_disjoint(&self, other: &ComponentSet) -> bool {
        self.0.is_disjoint(&other.0)
    }
    pub fn intersection<'a>(&'a self, rhs: &'a ComponentSet) -> impl Iterator<Item = usize> + 'a {
        self.0.intersection(&rhs.0)
    }
}
#[derive(Serialize, Deserialize)]
struct ComponentSetSerialized(u64, Vec<u8>);
impl Serialize for ComponentSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        ComponentSetSerialized(self.0.len() as u64, self.0.clone().into_bit_vec().to_bytes()).serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for ComponentSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let css = ComponentSetSerialized::deserialize(deserializer)?;
        let mut bv = BitVec::from_bytes(&css.1);
        bv.truncate(css.0 as usize);

        Ok(ComponentSet(BitSet::from_bit_vec(bv)))
    }
}
