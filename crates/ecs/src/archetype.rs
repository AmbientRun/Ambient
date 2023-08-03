use std::cell::UnsafeCell;

use ambient_native_std::sparse_vec::SparseVec;
use atomic_refcell::AtomicRefCell;

use super::*;
use crate::{
    component_traits::{ComponentBuffer, IComponentBuffer},
    ComponentEntry,
};

pub(super) struct ArchComponentData(UnsafeCell<Box<dyn IComponentBuffer>>);
impl Clone for ArchComponentData {
    fn clone(&self) -> Self {
        Self(UnsafeCell::new(
            { unsafe { &**self.0.get() } }.clone_boxed(),
        ))
    }
}

#[derive(Clone)]
pub(super) struct ArchComponent {
    pub(super) component: ComponentDesc,
    pub(super) data: ArchComponentData,
    pub(super) changes: AtomicRefCell<FramedEvents<EntityId>>,
    pub(super) content_versions: AtomicRefCell<Vec<u64>>,
    /// Content version doesn't change when an entity is moved from one archetype to another
    pub max_content_version: CloneableAtomicU64,
    /// Data version is like content_version, except it does update also when entities are moved in
    pub data_version: CloneableAtomicU64,
}

impl ArchComponent {
    pub fn new(component_buffer: Box<dyn IComponentBuffer>) -> Self {
        Self {
            component: component_buffer.desc(),
            data: ArchComponentData(UnsafeCell::new(component_buffer)),
            changes: AtomicRefCell::new(FramedEvents::new()),
            content_versions: AtomicRefCell::new(Vec::new()),
            max_content_version: CloneableAtomicU64::new(0),
            data_version: CloneableAtomicU64::new(0),
        }
    }

    pub(crate) fn on_write(&self, id: EntityId, index: usize, frame: u64) {
        self.changes.borrow_mut().add_event(id);
        // These do not depend on self ordering
        self.max_content_version.0.store(frame, Ordering::Relaxed);
        self.data_version.0.fetch_add(1, Ordering::Relaxed);
        self.set_content_version(index, frame, 1);
    }

    fn set_content_version(&self, index: usize, frame: u64, count: usize) {
        let mut content_versions = self.content_versions.borrow_mut();
        if content_versions.len() < index + count {
            content_versions.resize(index + count, frame);
        }
        for i in 0..count {
            content_versions[index + i] = frame;
        }
    }

    /// Content version doesn't change when an entity is moved from one archetype to another
    pub fn get_content_version(&self, index: usize) -> u64 {
        self.content_versions.borrow()[index]
    }

    fn reset_events(&mut self) {
        *self.changes.borrow_mut() = FramedEvents::new();
    }
}

#[derive(Clone)]
pub(super) struct MoveComponent {
    pub data: ComponentEntry,
    pub version: u64,
}
#[derive(Clone)]
pub struct EntityMoveData {
    content: SparseVec<MoveComponent>,
    pub active_components: ComponentSet,
}
impl EntityMoveData {
    fn new(active_components: ComponentSet) -> Self {
        Self {
            content: SparseVec::new(),
            active_components,
        }
    }
    pub(super) fn from_entity_data(entity_data: Entity, version: u64) -> Self {
        let mut s = Self::new(entity_data.active_components.clone());
        for data in entity_data {
            s.content
                .set(data.desc().index() as _, MoveComponent { data, version });
        }
        s
    }

    pub fn components(&self) -> Vec<ComponentDesc> {
        self.content.iter().map(|x| x.data.desc()).collect_vec()
    }

    pub fn set(&mut self, entry: ComponentEntry, version: u64) {
        let desc = entry.desc();
        let index = entry.desc().index();
        self.content.set(
            index as _,
            MoveComponent {
                data: entry,
                version,
            },
        );
        self.active_components.insert(desc);
    }

    pub fn remove(&mut self, component_index: usize) {
        self.active_components.remove_by_index(component_index);
        self.content.remove(component_index);
    }
}
impl From<EntityMoveData> for Entity {
    fn from(data: EntityMoveData) -> Self {
        let mut ed = Entity::new();
        for comp in data.content.into_iter() {
            ed.set_entry(comp.data);
        }
        ed
    }
}

/// Debug information of an archetype
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ArchetypeInfo {
    components: Vec<ComponentDesc>,
    entities: Vec<EntityId>,
}

pub type ArchetypeId = usize;

#[derive(Clone)]
pub struct Archetype {
    pub id: ArchetypeId,
    pub(super) entity_indices_to_ids: Vec<EntityId>,
    pub(super) components: SparseVec<ArchComponent>,
    pub(super) active_components: ComponentSet,
    pub(super) movein_events: FramedEvents<EntityId>,
    pub(super) moveout_events: FramedEvents<(EntityId, Entity)>,
    pub(super) query_markers: AtomicRefCell<Vec<u64>>,
}
impl Archetype {
    pub(super) fn new(arch_id: ArchetypeId, components: Vec<ComponentDesc>) -> Self {
        let mut arch_components = SparseVec::new();
        let mut active_components = ComponentSet::new();
        for &component in &components {
            arch_components.set(
                component.index() as usize,
                ArchComponent::new(component.create_buffer()),
            );
            active_components.insert(component);
        }
        Self {
            id: arch_id,
            entity_indices_to_ids: Vec::new(),
            components: arch_components,
            active_components,
            movein_events: FramedEvents::new(),
            moveout_events: FramedEvents::new(),
            query_markers: Default::default(),
        }
    }
    pub fn entity_count(&self) -> usize {
        self.entity_indices_to_ids.len()
    }
    pub fn get_entity_id_from_index(&self, index: usize) -> EntityId {
        self.entity_indices_to_ids[index]
    }
    pub fn next_index(&self) -> usize {
        self.entity_indices_to_ids.len()
    }
    pub fn write(&self, id: EntityId, index: usize, entity: Entity, version: u64) {
        for comp in entity {
            let arch_comp = self
                .components
                .get(comp.index() as _)
                .expect("Entity does not fit archetype");

            (unsafe { &mut **arch_comp.data.0.get() }).set(index, comp);
            arch_comp.on_write(id, index, version);
        }
    }

    pub fn movein(&mut self, ids: Vec<EntityId>, entity: EntityMoveData) {
        let index = self.entity_indices_to_ids.len();
        self.entity_indices_to_ids.extend(ids.iter().cloned());
        self.query_markers
            .borrow_mut()
            .resize(self.entity_indices_to_ids.len(), 0);
        for comp in entity.content.into_iter() {
            let arch_comp = self
                .components
                .get_mut(comp.data.index() as _)
                .expect("Entity does not fit archetype");
            (unsafe { &mut **arch_comp.data.0.get() }).append_cloned(comp.data, ids.len());
            arch_comp
                .max_content_version
                .0
                .fetch_max(comp.version, Ordering::Relaxed);
            arch_comp.set_content_version(index, comp.version, ids.len());
            arch_comp
                .changes
                .borrow_mut()
                .add_events(ids.iter().cloned());
            arch_comp.data_version.0.fetch_add(1, Ordering::Relaxed);
        }
        self.movein_events.add_events(ids.iter().cloned());
    }

    fn swap_remove_quiet(&mut self, index: usize, version: u64) -> EntityMoveData {
        self.entity_indices_to_ids.swap_remove(index);
        self.query_markers.borrow_mut().swap_remove(index);
        let mut entity_data = EntityMoveData::new(self.active_components.clone());

        for arch_comp in self.components.iter_mut() {
            let value = (unsafe { &mut **arch_comp.data.0.get() }).swap_remove_index(index);
            let content_version = arch_comp.content_versions.borrow_mut().swap_remove(index);
            entity_data.content.set(
                value.index() as _,
                MoveComponent {
                    data: value,
                    version: content_version,
                },
            );
            arch_comp
                .max_content_version
                .0
                .store(version, Ordering::Relaxed);
            arch_comp.data_version.0.fetch_add(1, Ordering::Relaxed);
        }

        entity_data
    }

    pub fn moveout(&mut self, index: usize, entity: EntityId, version: u64) -> EntityMoveData {
        let entity_data = self.swap_remove_quiet(index, version);
        self.moveout_events
            .add_event((entity, entity_data.clone().into()));
        entity_data
    }

    pub fn get_component_buffer<T: ComponentValue>(
        &self,
        component: Component<T>,
    ) -> Option<&ComponentBuffer<T>> {
        Some(
            self.get_component_buffer_untyped(component.desc())?
                .as_any()
                .downcast_ref()
                .unwrap(),
        )
    }

    pub fn get_component_buffer_untyped(
        &self,
        component: ComponentDesc,
    ) -> Option<&dyn IComponentBuffer> {
        if let Some(component) = self.components.get(component.index() as _) {
            Some(unsafe { &**component.data.0.get() })
        } else {
            None
        }
    }

    pub fn replace_with_entry(
        &mut self,
        id: EntityId,
        index: usize,
        entry: ComponentEntry,
        version: u64,
    ) -> Result<ComponentEntry, ECSError> {
        match self.get_arch_component_mut(entry.desc()) {
            Some(d) => {
                d.on_write(id, index, version);
                Ok(d.data.0.get_mut().set(index, entry))
            }
            None => Err(ECSError::EntityDoesntHaveComponent {
                component_index: entry.desc().index() as usize,
                name: entry.path(),
            }),
        }
    }

    fn get_arch_component_mut(&mut self, component: ComponentDesc) -> Option<&mut ArchComponent> {
        if let Some(component) = self.components.get_mut(component.index() as _) {
            Some(&mut *component)
        } else {
            None
        }
    }

    pub fn get_component<T: ComponentValue>(
        &self,
        entity_ix: usize,
        component: Component<T>,
    ) -> Option<&T> {
        self.get_component_buffer(component)
            .map(|buf| &buf.data[entity_ix])
    }

    pub fn get_component_mut<T: ComponentValue>(
        &self,
        entity_ix: usize,
        entity_id: EntityId,
        component: Component<T>,
        version: u64,
    ) -> Option<&mut T> {
        if let Some(arch_comp) = &self.components.get(component.index() as _) {
            arch_comp.on_write(entity_id, entity_ix, version);
            let x = unsafe { &mut **arch_comp.data.0.get() };
            x.as_mut_any()
                .downcast_mut::<ComponentBuffer<T>>()
                .map(|x| &mut x.data[entity_ix])
        } else {
            None
        }
    }

    #[allow(clippy::borrowed_box)]
    pub fn set_component_raw(
        &self,
        entity_ix: usize,
        entity_id: EntityId,
        entry: ComponentEntry,
        version: u64,
    ) -> bool {
        if let Some(arch_comp) = &self.components.get(entry.index() as usize) {
            arch_comp.on_write(entity_id, entity_ix, version);
            let buffer = unsafe { &mut **arch_comp.data.0.get() };
            buffer.set(entity_ix, entry);
            true
        } else {
            false
        }
    }

    #[ambient_profiling::function]
    pub fn next_frame(&mut self) {
        self.movein_events.next_frame();
        self.moveout_events.next_frame();
        for comp in self.components.iter() {
            comp.changes.borrow_mut().next_frame();
        }
    }
    pub fn get_component_content_version(&self, loc: EntityLocation, index: u32) -> Option<u64> {
        self.components
            .get(index as _)
            .map(|arch_comp| arch_comp.get_content_version(loc.index))
    }
    /// Content version doesn't change when an entity is moved
    pub fn get_component_max_content_version(&self, component: ComponentDesc) -> Option<u64> {
        self.components
            .get(component.index() as _)
            .map(|arch_comp| arch_comp.max_content_version.0.load(Ordering::Acquire))
    }
    /// Data version is like get_component_content_version except it always updates
    pub fn get_component_data_version(&self, component: ComponentDesc) -> Option<u64> {
        self.components
            .get(component.index() as _)
            .map(|arch_comp| arch_comp.data_version.0.load(Ordering::Acquire))
    }

    /// This returns true if the value hasn't been set for this entity before. I.e.:
    /// mark(5, 3) -> false
    /// mark(5, 3) -> true
    /// mark(5, 4) -> false
    pub(crate) fn query_mark(&self, index: usize, value: u64) -> bool {
        let mut marks = self.query_markers.borrow_mut();
        let cell = &mut marks[index];
        let changed = *cell != value;
        *cell = value;
        changed
    }

    pub(super) fn reset_events(&mut self) {
        self.movein_events = FramedEvents::new();
        self.moveout_events = FramedEvents::new();
        for comp in self.components.iter_mut() {
            comp.reset_events();
        }
    }

    pub fn dump_info(&self) -> ArchetypeInfo {
        ArchetypeInfo {
            components: self.components.iter().map(|v| v.component).collect_vec(),
            entities: self.entity_indices_to_ids.clone(),
        }
    }

    pub fn dump(&self, f: &mut dyn std::io::Write) {
        writeln!(
            f,
            "Archetype id: {} ({} entities)",
            self.id,
            self.entity_count()
        )
        .unwrap();
        for component in self.components.iter() {
            let desc = component.component;
            writeln!(
                f,
                "  Component {}: {} changes",
                desc.path(),
                component.changes.borrow().n_events()
            )
            .unwrap();
        }
        for i in 0..self.entity_count() {
            self.dump_entity(i, 2, f);
        }
    }
    pub fn dump_entity(&self, entity_ix: usize, indent: usize, f: &mut dyn std::io::Write) {
        let id = self.entity_indices_to_ids[entity_ix];
        let indent = format!("{:indent$}", "", indent = indent);
        writeln!(
            f,
            "{}Entity id={} loc={}:{}",
            indent, id, self.id, entity_ix
        )
        .unwrap();
        for component in self.components.iter() {
            let comp = unsafe { &mut **component.data.0.get() };
            let value = comp.dump_index(entity_ix);
            let value = value.split('\n').collect_vec();
            let content_version = component.get_content_version(entity_ix);
            let desc = comp.desc();
            let name = desc.path();

            if value.len() == 1 {
                writeln!(
                    f,
                    "{}  {}(v{}): {}",
                    indent, name, content_version, value[0]
                )
                .unwrap();
            } else {
                writeln!(f, "{indent}  {name}(v{content_version}):").unwrap();
                for row in &value {
                    writeln!(f, "{indent}  | {row}").unwrap();
                }
            }
        }
    }

    /// # Safety
    /// TODO
    pub fn dump_entity_to_yml(&self, entity_ix: usize) -> (String, yaml_rust::yaml::Hash) {
        let id = self.entity_indices_to_ids[entity_ix];
        let mut res = yaml_rust::yaml::Hash::new();
        for component in self.components.iter() {
            let comp = unsafe { &mut **component.data.0.get() };
            let value = comp.dump_index(entity_ix);
            let path = comp.desc().path();
            res.insert(
                yaml_rust::yaml::Yaml::String(path),
                yaml_rust::yaml::Yaml::String(value),
            );
        }
        (format!("id={} loc={}:{}", id, self.id, entity_ix), res)
    }
}
