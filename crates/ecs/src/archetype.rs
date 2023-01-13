use std::cell::UnsafeCell;

use atomic_refcell::AtomicRefCell;
use elements_std::sparse_vec::SparseVec;

use super::*;

pub(super) struct ArchComponentData(UnsafeCell<Box<dyn IComponentBuffer>>);
impl Clone for ArchComponentData {
    fn clone(&self) -> Self {
        Self(UnsafeCell::new({ unsafe { &**self.0.get() } }.clone_boxed()))
    }
}

#[derive(Clone)]
pub(super) struct ArchComponent {
    pub(super) component: Box<dyn IComponent>,
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
            component: component_buffer.component_boxed(),
            data: ArchComponentData(UnsafeCell::new(component_buffer)),
            changes: AtomicRefCell::new(FramedEvents::new()),
            content_versions: AtomicRefCell::new(Vec::new()),
            max_content_version: CloneableAtomicU64::new(0),
            data_version: CloneableAtomicU64::new(0),
        }
    }

    fn on_write(&self, id: EntityId, index: usize, frame: u64) {
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
    pub data: Box<dyn IComponentBuffer>,
    pub version: u64,
}
#[derive(Clone)]
pub struct EntityMoveData {
    content: SparseVec<MoveComponent>,
    pub active_components: ComponentSet,
}
impl EntityMoveData {
    fn new(active_components: ComponentSet) -> Self {
        Self { content: SparseVec::new(), active_components }
    }
    pub(super) fn from_entity_data(entity_data: EntityData, version: u64) -> Self {
        let mut s = Self::new(entity_data.active_components.clone());
        for data in entity_data {
            s.content.set(data.get_component_index(), MoveComponent { data: data.to_buf(), version });
        }
        s
    }
    pub fn components(&self) -> Vec<Box<dyn IComponent>> {
        self.content.iter().map(|x| x.data.component_boxed()).collect_vec()
    }
    pub fn set(&mut self, unit: ComponentUnit, version: u64) {
        self.content.set(unit.get_component_index(), MoveComponent { data: unit.to_buf(), version });
        self.active_components.insert(unit.component());
    }
    pub fn remove(&mut self, component_index: usize) {
        self.active_components.remove_by_index(component_index);
        self.content.remove(component_index);
    }
}
impl From<EntityMoveData> for EntityData {
    fn from(data: EntityMoveData) -> Self {
        let mut ed = EntityData::new();
        for mut comp in data.content.into_iter() {
            assert_eq!(comp.data.len(), 1, "Attempt to produce a single entity from EntityMoveData containing more than one");
            ed.set_unit(comp.data.pop_unit());
        }
        ed
    }
}

/// Debug information of an archetype
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ArchetypeInfo {
    components: Vec<String>,
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
    pub(super) moveout_events: FramedEvents<(EntityId, EntityData)>,
}
impl Archetype {
    pub(super) fn new(arch_id: ArchetypeId, components: Vec<Box<dyn IComponent>>) -> Self {
        let mut arch_components = SparseVec::new();
        let mut active_components = ComponentSet::new();
        for component in &components {
            arch_components.set(component.get_index(), ArchComponent::new(component.create_buffer()));
            active_components.insert(component.as_ref());
        }
        Self {
            id: arch_id,
            entity_indices_to_ids: Vec::new(),
            components: arch_components,
            active_components,
            movein_events: FramedEvents::new(),
            moveout_events: FramedEvents::new(),
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
    pub fn write(&self, id: EntityId, index: usize, entity: EntityData, version: u64) {
        for comp in entity {
            let arch_comp = self.components.get(comp.get_component_index()).expect("Entity does not fit archetype");

            (unsafe { &mut **arch_comp.data.0.get() }).set(index, comp.value());
            arch_comp.on_write(id, index, version);
        }
    }

    pub fn movein(&mut self, ids: Vec<EntityId>, entity: EntityMoveData) {
        let index = self.entity_indices_to_ids.len();
        self.entity_indices_to_ids.extend(ids.iter().cloned());
        for comp in entity.content.into_iter() {
            let arch_comp = self.components.get_mut(comp.data.component_index()).expect("Entity does not fit archetype");
            (unsafe { &mut **arch_comp.data.0.get() }).append(comp.data, ids.len());
            arch_comp.max_content_version.0.fetch_max(comp.version, Ordering::Relaxed);
            arch_comp.set_content_version(index, comp.version, ids.len());
            arch_comp.changes.borrow_mut().add_events(ids.iter().cloned());
            arch_comp.data_version.0.fetch_add(1, Ordering::Relaxed);
        }
        self.movein_events.add_events(ids.iter().cloned());
    }
    fn swap_remove_quiet(&mut self, index: usize, version: u64) -> EntityMoveData {
        self.entity_indices_to_ids.swap_remove(index);
        let mut entity_data = EntityMoveData::new(self.active_components.clone());

        for arch_comp in self.components.iter_mut() {
            let value = (unsafe { &mut **arch_comp.data.0.get() }).swap_remove_index(index);
            let content_version = arch_comp.content_versions.borrow_mut().swap_remove(index);
            entity_data.content.set(value.component_index(), MoveComponent { data: value, version: content_version });
            arch_comp.max_content_version.0.store(version, Ordering::Relaxed);
            arch_comp.data_version.0.fetch_add(1, Ordering::Relaxed);
        }

        entity_data
    }
    pub fn moveout(&mut self, index: usize, entity: EntityId, version: u64) -> EntityMoveData {
        let entity_data = self.swap_remove_quiet(index, version);
        self.moveout_events.add_event((entity, entity_data.clone().into()));
        entity_data
    }
    pub fn get_component_buffer<T: ComponentValue>(&self, component: Component<T>) -> Option<&ComponentBuffer<T>> {
        if let Some(component) = &self.components.get(component.get_index()) {
            let x = unsafe { &mut **component.data.0.get() };
            x.as_any().downcast_ref::<ComponentBuffer<T>>()
        } else {
            None
        }
    }
    pub fn get_component<T: ComponentValue>(&self, entity_ix: usize, component: Component<T>) -> Option<&T> {
        self.get_component_buffer(component).map(|buf| &buf.data[entity_ix])
    }
    pub fn get_component_mut<T: ComponentValue>(
        &self,
        entity_ix: usize,
        entity_id: EntityId,
        component: Component<T>,
        version: u64,
    ) -> Option<&mut T> {
        if let Some(arch_comp) = &self.components.get(component.get_index()) {
            arch_comp.on_write(entity_id, entity_ix, version);
            let x = unsafe { &mut **arch_comp.data.0.get() };
            x.as_mut_any().downcast_mut::<ComponentBuffer<T>>().map(|x| &mut x.data[entity_ix])
        } else {
            None
        }
    }
    #[allow(clippy::borrowed_box)]
    pub fn set_component_raw(
        &self,
        entity_ix: usize,
        entity_id: EntityId,
        component: &dyn IComponent,
        value: &Box<dyn ComponentValueBase>,
        version: u64,
    ) -> bool {
        if let Some(arch_comp) = &self.components.get(component.get_index()) {
            arch_comp.on_write(entity_id, entity_ix, version);
            let x = unsafe { &mut **arch_comp.data.0.get() };
            x.set(entity_ix, value);
            true
        } else {
            false
        }
    }

    pub fn next_frame(&mut self) {
        self.movein_events.next_frame();
        self.moveout_events.next_frame();
        for comp in self.components.iter() {
            comp.changes.borrow_mut().next_frame();
        }
    }
    pub fn get_component_content_version(&self, loc: EntityLocation, component: &dyn IComponent) -> Option<u64> {
        self.components.get(component.get_index()).map(|arch_comp| arch_comp.get_content_version(loc.index))
    }
    /// Content version doesn't change when an entity is moved
    pub fn get_component_max_content_version(&self, component: &dyn IComponent) -> Option<u64> {
        self.components.get(component.get_index()).map(|arch_comp| arch_comp.max_content_version.0.load(Ordering::Acquire))
    }
    /// Data version is like get_component_content_version except it always updates
    pub fn get_component_data_version(&self, component: &dyn IComponent) -> Option<u64> {
        self.components.get(component.get_index()).map(|arch_comp| arch_comp.data_version.0.load(Ordering::Acquire))
    }

    pub(super) fn reset_events(&mut self) {
        self.movein_events = FramedEvents::new();
        self.moveout_events = FramedEvents::new();
        for comp in self.components.iter_mut() {
            comp.reset_events();
        }
    }

    pub fn dump_info(&self) -> ArchetypeInfo {
        let idx_to_id = with_component_registry(|r| r.idx_to_id().clone());

        ArchetypeInfo {
            components: self
                .components
                .iter()
                .map(|v| {
                    let idx = v.component.get_index();
                    idx_to_id.get(&idx).cloned().unwrap_or_else(|| idx.to_string())
                })
                .collect_vec(),
            entities: self.entity_indices_to_ids.clone(),
        }
    }

    pub fn dump(&self, f: &mut dyn std::io::Write) {
        let idx_to_id = with_component_registry(|r| r.idx_to_id().clone());
        writeln!(f, "Archetype id: {} ({} entities)", self.id, self.entity_count()).unwrap();
        for component in self.components.iter() {
            let comp = unsafe { &mut **component.data.0.get() };
            let idx = comp.component_index();
            writeln!(
                f,
                "  Component {}: {} changes",
                idx_to_id.get(&idx).cloned().unwrap_or_else(|| idx.to_string()),
                component.changes.borrow().n_events()
            )
            .unwrap();
        }
        for i in 0..self.entity_count() {
            self.dump_entity(i, 2, &idx_to_id, f);
        }
    }
    pub fn dump_entity(&self, entity_ix: usize, indent: usize, idx_to_id: &HashMap<usize, String>, f: &mut dyn std::io::Write) {
        let id = self.entity_indices_to_ids[entity_ix];
        let indent = format!("{:indent$}", "", indent = indent);
        writeln!(f, "{}Entity id={} loc={}:{}", indent, id, self.id, entity_ix).unwrap();
        for component in self.components.iter() {
            let comp = unsafe { &mut **component.data.0.get() };
            let value = comp.dump_index(entity_ix);
            let value = value.split('\n').collect_vec();
            let content_version = component.get_content_version(entity_ix);
            let idx = comp.component_index();
            let name = idx_to_id.get(&idx).cloned().unwrap_or_else(|| idx.to_string());
            if value.len() == 1 {
                writeln!(f, "{}  {}(v{}): {}", indent, name, content_version, value[0]).unwrap();
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
        let idx_to_id = with_component_registry(|r| r.idx_to_id().clone());
        let id = self.entity_indices_to_ids[entity_ix];
        let mut res = yaml_rust::yaml::Hash::new();
        for component in self.components.iter() {
            let comp = unsafe { &mut **component.data.0.get() };
            let value = comp.dump_index(entity_ix);
            let idx = comp.component_index();
            res.insert(
                yaml_rust::yaml::Yaml::String(idx_to_id.get(&idx).cloned().unwrap_or_else(|| idx.to_string())),
                yaml_rust::yaml::Yaml::String(value),
            );
        }
        (format!("id={} loc={}:{}", id, self.id, entity_ix), res)
    }
}
