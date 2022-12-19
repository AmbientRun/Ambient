use std::fmt::Display;

use itertools::Itertools;

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchetypeFilter {
    components: ComponentSet,
    not_components: ComponentSet,
}
impl ArchetypeFilter {
    pub fn new() -> Self {
        Self { components: ComponentSet::new(), not_components: ComponentSet::new() }
    }
    pub fn incl_ref(mut self, component: &dyn IComponent) -> Self {
        self.components.insert(component);
        self
    }
    pub fn incl<T: IComponent>(self, component: T) -> Self {
        self.incl_ref(&component)
    }
    pub fn excl_ref(mut self, component: &dyn IComponent) -> Self {
        self.not_components.insert(component);
        self
    }
    pub fn excl<T: IComponent>(self, component: T) -> Self {
        self.excl_ref(&component)
    }
    pub(crate) fn matches(&self, components: &ComponentSet) -> bool {
        components.is_superset(&self.components) && components.is_disjoint(&self.not_components)
    }
    pub fn matches_entity(&self, world: &World, id: EntityId) -> bool {
        if let Some(loc) = world.locs.get(id) {
            let arch = world.archetypes.get(loc.archetype).expect("Archetype doesn't exist");
            self.matches(&arch.active_components)
        } else {
            false
        }
    }
    pub fn matches_archetype(&self, arch: &Archetype) -> bool {
        self.matches(&arch.active_components)
    }
    pub fn iter_archetypes<'a>(&self, world: &'a World) -> impl Iterator<Item = &'a Archetype> {
        self.iter_by_archetypes(&world.archetypes)
    }
    fn iter_by_archetypes<'a>(&self, archetypes: &'a [Archetype]) -> impl Iterator<Item = &'a Archetype> {
        let f = self.clone();
        archetypes.iter().filter(move |arch| f.matches(&arch.active_components))
    }
    pub fn iter_entities<'a>(&self, world: &'a World) -> impl Iterator<Item = EntityAccessor> + 'a {
        self.iter_by_archetypes(&world.archetypes)
            .flat_map(|arch| arch.entity_indices_to_ids.iter().map(move |&id| EntityAccessor::World { id }))
    }
}
impl Default for ArchetypeFilter {
    fn default() -> Self {
        Self::new()
    }
}

pub trait ComponentsTuple<'a>: Send + Sync {
    fn write_component_ids(&self, set: &mut ComponentSet);
    fn get_change_filtered(&self, out: &mut Vec<Box<dyn IComponent>>);
    type Data;
    type DataMut;
    type DataCloned;
    fn get_data(&self, world: &'a World, acc: &EntityAccessor) -> Self::Data;
    fn get_data_mut(&self, world: &'a World, acc: &EntityAccessor) -> Self::DataMut;
    fn get_data_cloned(&self, world: &'a World, acc: &EntityAccessor) -> Self::DataCloned;
}
pub trait ComponentsTupleAppend<T: ComponentValue> {
    type Output;
    fn append(&self, component: Component<T>) -> Self::Output;
}

// From: https://stackoverflow.com/questions/56697029/is-there-a-way-to-impl-trait-for-a-tuple-that-may-have-any-number-elements
macro_rules! tuple_impls {
    ( $( $name:ident )+ ) => {
        impl<'a, $($name: ComponentValue),+> ComponentsTuple<'a> for ($(Component<$name>,)+) {
            fn write_component_ids(&self, set: &mut ComponentSet) {
                #[allow(non_snake_case)]
                let ($($name,)+) = self;
                $(set.insert($name);)+
            }
            fn get_change_filtered(&self, out: &mut Vec<Box<dyn IComponent>>) {
                #[allow(non_snake_case)]
                let ($($name,)+) = self;
                $(
                    if $name.is_change_filter() {
                        out.push(Box::new(*$name));
                    }
                )+
            }
            type Data = ($(&'a $name,)+);
            type DataMut = ($(&'a mut $name,)+);
            type DataCloned = ($($name,)+);
            fn get_data(&self, world: &'a World, acc: &EntityAccessor) -> Self::Data {
                #[allow(non_snake_case)]
                let ($($name,)+) = self;
                ($(acc.get(world, *$name),)+)
            }
            fn get_data_mut(&self, world: &'a World, acc: &EntityAccessor) -> Self::DataMut {
                #[allow(non_snake_case)]
                let ($($name,)+) = self;
                ($(acc.get_mut(world, *$name),)+)
            }
            fn get_data_cloned(&self, world: &'a World, acc: &EntityAccessor) -> Self::DataCloned {
                #[allow(non_snake_case)]
                let ($($name,)+) = self;
                ($(acc.get(world, *$name).clone(),)+)
            }
        }
        impl<T: ComponentValue, $($name: ComponentValue),+> ComponentsTupleAppend<T> for ($(Component<$name>,)+) {
            type Output = ($(Component<$name>,)+ Component<T>);
            fn append(&self, component: Component<T>) -> Self::Output {
                #[allow(non_snake_case)]
                let ($($name,)+) = self;
                ($(*$name,)+ component)
            }
        }
    };
}
tuple_impls! { A }
tuple_impls! { A B }
tuple_impls! { A B C }
tuple_impls! { A B C D }
tuple_impls! { A B C D E }
tuple_impls! { A B C D E F }
tuple_impls! { A B C D E F G }
tuple_impls! { A B C D E F G H }
tuple_impls! { A B C D E F G H I }

impl<'a, T: ComponentValue> ComponentsTuple<'a> for Component<T> {
    fn write_component_ids(&self, set: &mut ComponentSet) {
        set.insert(self);
    }

    fn get_change_filtered(&self, out: &mut Vec<Box<dyn IComponent>>) {
        if self.is_change_filter() {
            out.push(Box::new(*self))
        }
    }

    type Data = &'a T;

    type DataMut = &'a mut T;

    type DataCloned = T;

    fn get_data(&self, world: &'a World, acc: &EntityAccessor) -> Self::Data {
        acc.get(world, *self)
    }

    fn get_data_mut(&self, world: &'a World, acc: &EntityAccessor) -> Self::DataMut {
        acc.get_mut(world, *self)
    }

    fn get_data_cloned(&self, world: &'a World, acc: &EntityAccessor) -> Self::DataCloned {
        acc.get(world, *self).clone()
    }
}
// -- Reduce compile times
// -- If the user uses such a large query they may consider splitting up the
// query
// tuple_impls! { A B C D E F G H I J }
// tuple_impls! { A B C D E F G H I J K }
// tuple_impls! { A B C D E F G H I J K L }

impl<'a> ComponentsTuple<'a> for () {
    fn write_component_ids(&self, _: &mut ComponentSet) {}
    fn get_change_filtered(&self, _: &mut Vec<Box<dyn IComponent>>) {}
    type Data = ();
    type DataMut = ();
    type DataCloned = ();
    fn get_data(&self, _: &World, _: &EntityAccessor) -> Self::Data {}
    fn get_data_mut(&self, _: &World, _: &EntityAccessor) -> Self::DataMut {}
    fn get_data_cloned(&self, _: &World, _: &EntityAccessor) -> Self::DataCloned {}
}
impl<T: ComponentValue> ComponentsTupleAppend<T> for () {
    type Output = (Component<T>,);
    fn append(&self, component: Component<T>) -> Self::Output {
        (component,)
    }
}

#[derive(Debug, Clone, Copy)]
struct EntityMark {
    value: u64,
    gen: i32,
}
#[derive(Debug, Clone)]
struct EntityMarker {
    marks: Vec<Vec<EntityMark>>,
}
impl EntityMarker {
    fn new() -> Self {
        Self { marks: Vec::new() }
    }
    fn prepare_for_query(&mut self, world: &World) {
        if self.marks.len() < world.locs.allocated.len() {
            self.marks.resize(world.locs.allocated.len(), Vec::new());
        }
        for (i, vals) in world.locs.allocated.iter().enumerate() {
            if self.marks[i].len() < vals.len() {
                self.marks[i].resize(vals.len(), EntityMark { value: 0, gen: -1 });
            }
        }
    }
    /// This returns true if the value hasn't been set for this entity before. I.e.:
    /// mark(5, 3) -> false
    /// mark(5, 3) -> true
    /// mark(5, 4) -> false
    fn mark(&mut self, id: EntityId, value: u64) -> bool {
        let cell = &mut self.marks[id.namespace as usize][id.id];
        let changed = cell.gen != id.gen || cell.value != value;
        cell.value = value;
        cell.gen = id.gen;
        changed
    }
}

#[derive(Debug, Clone)]
pub struct QueryState {
    inited: bool,
    change_readers: SparseVec<SparseVec<FramedEventsReader<EntityId>>>,
    movein_readers: SparseVec<FramedEventsReader<EntityId>>,
    moveout_readers: SparseVec<FramedEventsReader<(EntityId, EntityData)>>,
    processed: EntityMarker,
    processed_ticker: u64,
    spawned: EntityMarker,
    world_version: u64,
    entities: Vec<EntityAccessor>,
}
impl QueryState {
    pub fn new() -> Self {
        Self {
            inited: false,
            change_readers: SparseVec::new(),
            movein_readers: SparseVec::new(),
            moveout_readers: SparseVec::new(),
            processed: EntityMarker::new(),
            processed_ticker: 1,
            spawned: EntityMarker::new(),
            world_version: 0,
            entities: Vec::new(),
        }
    }
    pub(super) fn get_change_reader(&mut self, arch: usize, comp: usize) -> &mut FramedEventsReader<EntityId> {
        let a = self.change_readers.get_mut_or_insert_with(arch, SparseVec::new);
        a.get_mut_or_insert_with(comp, FramedEventsReader::new)
    }
    pub(super) fn get_movein_reader(&mut self, arch: usize) -> &mut FramedEventsReader<EntityId> {
        self.movein_readers.get_mut_or_insert_with(arch, FramedEventsReader::new)
    }
    pub(super) fn get_moveout_reader(&mut self, arch: usize) -> &mut FramedEventsReader<(EntityId, EntityData)> {
        self.moveout_readers.get_mut_or_insert_with(arch, FramedEventsReader::new)
    }
    pub(super) fn prepare_for_query(&mut self, world: &World) {
        self.processed.prepare_for_query(world);
        self.processed_ticker += 1;
        self.spawned.prepare_for_query(world);
    }
}

#[derive(Clone, Debug)]
pub enum QueryEvent {
    Frame,
    Changed { components: Vec<Box<dyn IComponent>> },
    Spawned,
    Despawned,
}
impl QueryEvent {
    pub fn is_frame(&self) -> bool {
        matches!(self, QueryEvent::Frame)
    }
    pub fn is_spawned(&self) -> bool {
        matches!(self, QueryEvent::Spawned)
    }
    pub fn is_despawned(&self) -> bool {
        matches!(self, QueryEvent::Despawned)
    }
}

#[derive(Debug, Clone)]
pub struct Query {
    pub filter: ArchetypeFilter,
    pub event: QueryEvent,
}

impl Query {
    pub fn new(filter: ArchetypeFilter) -> Self {
        Self { filter, event: QueryEvent::Frame }
    }

    pub fn all() -> Self {
        Self::new(ArchetypeFilter::new())
    }

    pub fn any_changed(components: Vec<Box<dyn IComponent>>) -> Self {
        let mut q = Self::all();
        for comp in components {
            q = q.when_changed_ref(comp.as_ref());
        }
        q
    }

    fn new_for_typed_query(component_ids: ComponentSet, changed_components: Vec<Box<dyn IComponent>>) -> Self {
        Query {
            filter: ArchetypeFilter { components: component_ids, not_components: ComponentSet::new() },
            event: if !changed_components.is_empty() { QueryEvent::Changed { components: changed_components } } else { QueryEvent::Frame },
        }
    }

    pub fn when_changed_ref(mut self, component: &dyn IComponent) -> Self {
        if let QueryEvent::Changed { components } = &mut self.event {
            components.push(component.clone_boxed());
        } else {
            self.event = QueryEvent::Changed { components: vec![component.clone_boxed()] };
        }
        self
    }

    pub fn when_changed<T: IComponent + 'static>(self, component: T) -> Self {
        self.when_changed_ref(&component)
    }
    pub fn incl_ref(mut self, component: &dyn IComponent) -> Self {
        self.filter = self.filter.incl_ref(component);
        self
    }
    pub fn incl<T: IComponent>(self, component: T) -> Self {
        self.incl_ref(&component)
    }
    pub fn excl_ref(mut self, component: &dyn IComponent) -> Self {
        self.filter = self.filter.excl_ref(component);
        self
    }
    pub fn excl<T: IComponent>(self, component: T) -> Self {
        self.excl_ref(&component)
    }
    pub fn optional_changed_ref(mut self, component: &dyn IComponent) -> Self {
        let event = std::mem::replace(&mut self.event, QueryEvent::Frame);
        self.event = match event {
            QueryEvent::Frame => QueryEvent::Changed { components: vec![component.clone_boxed()] },
            QueryEvent::Changed { mut components } => {
                components.push(component.clone_boxed());
                QueryEvent::Changed { components }
            }
            _ => {
                panic!("optional_changed can only be applied to Frame or Change queries (not Spawn or Despawn queries)")
            }
        };
        self
    }
    /// Changes to this component trigger the query, but the component is not required
    pub fn optional_changed<T: IComponent>(self, component: T) -> Self {
        self.optional_changed_ref(&component)
    }
    pub fn spawned(mut self) -> Self {
        self.event = QueryEvent::Spawned;
        self
    }
    pub fn despawned(mut self) -> Self {
        self.event = QueryEvent::Despawned;
        self
    }
    pub fn filter(mut self, filter: &ArchetypeFilter) -> Self {
        self.filter.components.union_with(&filter.components);
        self.filter.not_components.union_with(&filter.not_components);
        self
    }
    fn get_changed(&self, world: &World, state: &mut QueryState, components: &Vec<Box<dyn IComponent>>) {
        if !state.inited && !world.ignore_query_inits {
            for arch in self.filter.iter_by_archetypes(&world.archetypes) {
                for comp in components {
                    if let Some(arch_comp) = arch.components.get(comp.get_index()) {
                        let events = &*arch_comp.changes.borrow();
                        let read = state.get_change_reader(arch.id, comp.get_index());
                        read.move_to_end(events);
                    }
                }
            }
            return;
        }
        for arch in self.filter.iter_by_archetypes(&world.archetypes) {
            for comp in components {
                if let Some(arch_comp) = arch.components.get(comp.get_index()) {
                    let read = state.get_change_reader(arch.id, comp.get_index());
                    let events = &*arch_comp.changes.borrow();
                    for (_, &entity_id) in read.iter(events) {
                        if let Some(loc) = world.locs.get(entity_id) {
                            if loc.archetype == arch.id
                                && arch_comp.get_content_version(loc.index) > state.world_version
                                && state.processed.mark(entity_id, state.processed_ticker)
                            {
                                state.entities.push(EntityAccessor::World { id: entity_id });
                            }
                        }
                    }
                }
            }
        }
    }
    fn get_spawned(&self, world: &World, state: &mut QueryState) {
        if self.init_state_event_readers(world, state) {
            state.entities.extend(self.filter.iter_entities(world));
            for ea in state.entities.iter() {
                let id = ea.id();
                state.spawned.mark(id, 1);
            }
            return;
        }
        state.entities.clear();
        for arch in self.filter.iter_by_archetypes(&world.archetypes) {
            let read = state.get_movein_reader(arch.id);
            for (_, id) in read.iter(&arch.movein_events) {
                if let Some(loc) = world.locs.get(*id) {
                    if loc.archetype == arch.id {
                        let spawn = state.spawned.mark(*id, 1);
                        if spawn {
                            let process = state.processed.mark(*id, state.processed_ticker);
                            if process {
                                state.entities.push(EntityAccessor::World { id: *id });
                            }
                        }
                    }
                }
            }
            let read = state.get_moveout_reader(arch.id);
            for (_, (id, _)) in read.iter(&arch.moveout_events) {
                if !self.filter.matches_entity(world, *id) {
                    state.spawned.mark(*id, 0);
                }
            }
        }
    }
    fn get_despawned(&self, world: &World, state: &mut QueryState) {
        if self.init_state_event_readers(world, state) {
            return;
        }

        state.entities.clear();
        for arch in self.filter.iter_by_archetypes(&world.archetypes) {
            let read = state.get_moveout_reader(arch.id);
            for (event_id, (id, _)) in read.iter(&arch.moveout_events) {
                let next_matched = if let Some(loc) = world.locs.get(*id) {
                    self.filter.matches(&world.archetypes[loc.archetype].active_components)
                } else {
                    false
                };

                if !next_matched {
                    state.entities.push(EntityAccessor::Despawned { id: *id, archetype: arch.id, event_id });
                }
            }
        }
    }
    fn init_state_event_readers(&self, world: &World, state: &mut QueryState) -> bool {
        if state.inited || world.ignore_query_inits {
            return false;
        }
        for arch in self.filter.iter_by_archetypes(&world.archetypes) {
            let read_in = state.get_movein_reader(arch.id);
            read_in.move_to_end(&arch.movein_events);
            let read_out = state.get_moveout_reader(arch.id);
            read_out.move_to_end(&arch.moveout_events);
        }
        true
    }
    pub fn iter<'a>(&self, world: &'a World, state: Option<&'a mut QueryState>) -> Box<dyn Iterator<Item = EntityAccessor> + 'a> {
        if let QueryEvent::Frame = &self.event {
            return Box::new(self.filter.iter_entities(world));
        }

        let state = state.expect("Spawn/despawn/change queries must have a query state");
        if !self.event.is_frame() {
            state.prepare_for_query(world);
        }
        match &self.event {
            QueryEvent::Changed { components } => {
                self.get_spawned(world, state);
                self.get_changed(world, state, components);
            }
            QueryEvent::Spawned => self.get_spawned(world, state),
            QueryEvent::Despawned => self.get_despawned(world, state),
            _ => unreachable!(),
        };
        state.inited = true;
        state.world_version = world.version();
        Box::new(state.entities.iter().copied())
    }
    pub fn to_system<F: Fn(&Self, &mut World, &mut QueryState, &E) + Send + Sync + 'static, E: 'static>(
        self,
        update: F,
    ) -> Box<dyn System<E> + Sync + Send> {
        let mut state = QueryState::new();
        Box::new(FnSystem(Box::new(move |world, event| {
            update(&self, world, &mut state, event);
        })))
    }

    pub fn with_commands<F, E>(self, update: F) -> Box<dyn System<E>>
    where
        F: Fn(&Self, &mut World, Option<&mut QueryState>, &E, &mut Commands) + Send + Sync + 'static,
        E: 'static,
    {
        let mut state = QueryState::new();
        let mut commands = Commands::new();
        Box::new(FnSystem(Box::new(move |world, event| {
            update(&self, world, Some(&mut state), event, &mut commands);
            commands.soft_apply(world);
        })))
    }
    fn add_component<T: IComponent>(&mut self, query: &Self, component: T) {
        self.filter = query.filter.clone().incl(component);
        if query.event.is_spawned() {
            self.event = QueryEvent::Spawned;
        } else if query.event.is_despawned() {
            self.event = QueryEvent::Despawned;
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EntityAccessor {
    World { id: EntityId },
    Despawned { id: EntityId, archetype: usize, event_id: DBEventId },
}
impl EntityAccessor {
    pub fn id(&self) -> EntityId {
        match self {
            Self::World { id } => *id,
            Self::Despawned { id, .. } => *id,
        }
    }
    pub fn get<'a, T: ComponentValue>(&self, world: &'a World, component: Component<T>) -> &'a T {
        match self {
            Self::World { id } => world.get_ref(*id, component).unwrap(),
            Self::Despawned { archetype, event_id, .. } => {
                world.archetypes[*archetype].moveout_events.get(*event_id).unwrap().1.get_ref(component).unwrap()
            }
        }
    }
    pub fn get_mut<'a, T: ComponentValue>(&self, world: &'a World, component: Component<T>) -> &'a mut T {
        match self {
            Self::World { id } => world.get_mut_unsafe(*id, component).unwrap(),
            Self::Despawned { .. } => panic!("Can't mutate despawned entities"),
        }
    }
}

pub fn query<'a, R: ComponentsTuple<'a> + Clone + 'static>(read_components: R) -> TypedReadQuery<R> {
    TypedReadQuery::new(read_components)
}
pub fn query_mut<'a, RW: ComponentsTuple<'a> + Clone + 'static, R: ComponentsTuple<'a> + Clone + 'static>(
    read_write_components: RW,
    read_components: R,
) -> TypedReadWriteQuery<RW, R> {
    TypedReadWriteQuery::new(read_write_components, read_components)
}

pub struct TypedReadQuery<R> {
    read_components: R,
    pub query: Query,
}

impl<R> Debug for TypedReadQuery<R>
where
    R: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypedReadQuery").field("read_components", &self.read_components).finish_non_exhaustive()
    }
}

impl<'a, R: ComponentsTuple<'a> + Clone + 'static> TypedReadQuery<R> {
    pub fn new(read_components: R) -> Self {
        let mut component_ids = ComponentSet::new();
        read_components.write_component_ids(&mut component_ids);
        let mut changed_components = Vec::new();
        read_components.get_change_filtered(&mut changed_components);
        Self { query: Query::new_for_typed_query(component_ids, changed_components), read_components }
    }
    pub fn read<T: ComponentValue>(&self, component: Component<T>) -> TypedReadQuery<<R as ComponentsTupleAppend<T>>::Output>
    where
        R: ComponentsTupleAppend<T>,
        <R as ComponentsTupleAppend<T>>::Output: ComponentsTuple<'a> + Clone + 'static,
    {
        let mut q = TypedReadQuery::new(self.read_components.append(component));
        q.query.add_component(&self.query, component);
        q
    }
    pub fn filter(mut self, filter: &ArchetypeFilter) -> Self {
        self.query = self.query.filter(filter);
        self
    }
    pub fn incl<T: IComponent>(mut self, component: T) -> Self {
        self.query.filter = self.query.filter.incl(component);
        self
    }
    pub fn excl<T: IComponent>(mut self, component: T) -> Self {
        self.query.filter = self.query.filter.excl(component);
        self
    }
    /// Changes to this component trigger the query, but the component is not required
    pub fn optional_changed<T: IComponent>(mut self, component: T) -> Self {
        self.query = self.query.optional_changed(component);
        self
    }
    pub fn spawned(mut self) -> Self {
        self.query.event = QueryEvent::Spawned;
        self
    }
    pub fn despawned(mut self) -> Self {
        self.query.event = QueryEvent::Despawned;
        self
    }

    pub fn iter(
        &self,
        world: &'a World,
        state: Option<&'a mut QueryState>,
    ) -> impl Iterator<Item = (EntityId, <R as ComponentsTuple<'a>>::Data)> + 'a {
        let r = self.read_components.clone();
        self.query.iter(world, state).into_iter().map(move |acc| (acc.id(), r.get_data(world, &acc)))
    }
    pub fn iter_cloned(
        &self,
        world: &'a World,
        state: Option<&'a mut QueryState>,
    ) -> impl Iterator<Item = (EntityId, <R as ComponentsTuple<'a>>::DataCloned)> + 'a {
        let r = self.read_components.clone();
        self.query.iter(world, state).into_iter().map(move |acc| (acc.id(), r.get_data_cloned(world, &acc)))
    }
    pub fn collect_ids(&self, world: &'a World, state: Option<&'a mut QueryState>) -> Vec<EntityId> {
        self.query.iter(world, state).into_iter().map(move |acc| acc.id()).collect_vec()
    }
    pub fn collect_cloned(
        &self,
        world: &'a World,
        state: Option<&'a mut QueryState>,
    ) -> Vec<(EntityId, <R as ComponentsTuple<'a>>::DataCloned)> {
        self.iter_cloned(world, state).collect_vec()
    }
    // attempts to read the first item from the query if it exists; will discard
    // the other items
    pub fn read_one_cloned(
        &self,
        world: &'a World,
        state: Option<&'a mut QueryState>,
    ) -> Option<(EntityId, <R as ComponentsTuple<'a>>::DataCloned)> {
        self.iter_cloned(world, state).next()
    }
    pub fn to_system<F: FnMut(&Self, &mut World, Option<&mut QueryState>, &E) + Send + Sync + 'static, E: 'static>(
        self,
        update: F,
    ) -> DynSystem<E> {
        self.to_system_with_name("Unknown System", update)
    }
    pub fn to_system_with_name<F: FnMut(&Self, &mut World, Option<&mut QueryState>, &E) + Send + Sync + 'static, E: 'static>(
        self,
        name: &'static str,
        mut update: F,
    ) -> DynSystem<E> {
        let mut state = QueryState::new();
        Box::new(FnSystem(Box::new(move |world, event| {
            profiling::scope!(name);
            update(&self, world, Some(&mut state), event);
        })))
    }

    pub fn with_commands<F, E>(self, update: F) -> DynSystem<E>
    where
        F: Fn(&Self, &mut World, Option<&mut QueryState>, &E, &mut Commands) + Send + Sync + 'static,
        E: 'static,
    {
        let mut state = QueryState::new();
        let mut commands = Commands::new();
        Box::new(FnSystem(Box::new(move |world, event| {
            update(&self, world, Some(&mut state), event, &mut commands);
            commands.soft_apply(world);
        })))
    }
}

pub struct TypedReadWriteQuery<RW, R> {
    read_write_components: RW,
    read_components: R,
    query: Query,
}
impl<'a, RW: ComponentsTuple<'a> + Clone + 'static, R: ComponentsTuple<'a> + Clone + 'static> TypedReadWriteQuery<RW, R> {
    pub fn new(read_write_components: RW, read_components: R) -> Self {
        let mut write_set = ComponentSet::new();
        let mut read_set = ComponentSet::new();
        read_write_components.write_component_ids(&mut write_set);
        read_components.write_component_ids(&mut read_set);

        if let Some(id) = write_set.intersection(&read_set).next() {
            panic!("Non disjoint query component: {id}")
        }

        let mut component_ids = ComponentSet::new();
        read_write_components.write_component_ids(&mut component_ids);
        read_components.write_component_ids(&mut component_ids);
        let mut changed_components = Vec::new();
        read_write_components.get_change_filtered(&mut changed_components);
        read_components.get_change_filtered(&mut changed_components);
        Self { query: Query::new_for_typed_query(component_ids, changed_components), read_write_components, read_components }
    }
    pub fn read_write<T: ComponentValue>(&self, component: Component<T>) -> TypedReadWriteQuery<<RW as ComponentsTupleAppend<T>>::Output, R>
    where
        RW: ComponentsTupleAppend<T>,
        <RW as ComponentsTupleAppend<T>>::Output: ComponentsTuple<'a> + Clone + 'static,
    {
        let mut q = TypedReadWriteQuery::new(self.read_write_components.append(component), self.read_components.clone());
        q.query.add_component(&self.query, component);
        q
    }
    pub fn read<T: ComponentValue>(&self, component: Component<T>) -> TypedReadWriteQuery<RW, <R as ComponentsTupleAppend<T>>::Output>
    where
        R: ComponentsTupleAppend<T>,
        <R as ComponentsTupleAppend<T>>::Output: ComponentsTuple<'a> + Clone + 'static,
    {
        let mut q = TypedReadWriteQuery::new(self.read_write_components.clone(), self.read_components.append(component));
        q.query.add_component(&self.query, component);
        q
    }
    pub fn filter(mut self, filter: &ArchetypeFilter) -> Self {
        self.query.filter.components.union_with(&filter.components);
        self.query.filter.not_components.union_with(&filter.not_components);
        self
    }
    pub fn incl<T: IComponent>(mut self, component: T) -> Self {
        self.query.filter = self.query.filter.incl(component);
        self
    }
    pub fn excl<T: IComponent>(mut self, component: T) -> Self {
        self.query.filter = self.query.filter.excl(component);
        self
    }
    /// Changes to this component trigger the query, but the component is not required
    pub fn optional_changed<T: IComponent>(mut self, component: T) -> Self {
        self.query = self.query.optional_changed(component);
        self
    }
    pub fn spawned(mut self) -> Self {
        self.query.event = QueryEvent::Spawned;
        self
    }
    pub fn despawned(mut self) -> Self {
        self.query.event = QueryEvent::Despawned;
        self
    }

    pub fn iter(
        &self,
        world: &'a mut World,
        state: Option<&'a mut QueryState>,
    ) -> impl Iterator<Item = (EntityId, <RW as ComponentsTuple<'a>>::DataMut, <R as ComponentsTuple<'a>>::Data)> + 'a {
        let rw = self.read_write_components.clone();
        let r = self.read_components.clone();
        let world = &*world;
        self.query.iter(world, state).into_iter().map(move |acc| (acc.id(), rw.get_data_mut(world, &acc), r.get_data(world, &acc)))
    }
    pub fn to_system<F: Fn(&Self, &mut World, Option<&mut QueryState>, &E) + Send + Sync + 'static, E: 'static>(
        self,
        update: F,
    ) -> DynSystem<E> {
        self.to_system_with_name("Default", update)
    }
    pub fn to_system_with_name<F: Fn(&Self, &mut World, Option<&mut QueryState>, &E) + Send + Sync + 'static, E: 'static>(
        self,
        name: &'static str,
        update: F,
    ) -> DynSystem<E> {
        let mut state = QueryState::new();
        Box::new(FnSystem(Box::new(move |world, event| {
            profiling::scope!(name);
            update(&self, world, Some(&mut state), event);
        })))
    }

    pub fn with_commands<F, E>(self, update: F) -> DynSystem<E>
    where
        F: Fn(&Self, &mut World, Option<&mut QueryState>, &E, &mut Commands) + Sync + Send + 'static,
        E: 'static,
    {
        let mut state = QueryState::new();
        let mut commands = Commands::new();
        Box::new(FnSystem(Box::new(move |world, event| {
            update(&self, world, Some(&mut state), event, &mut commands);
            commands.soft_apply(world);
        }))) as Box<dyn System<E> + Send + Sync + 'static>
    }
}

pub struct FrameEvent;

pub trait System<E = FrameEvent>: Send + std::fmt::Debug {
    fn run(&mut self, world: &mut World, event: &E);
}

pub struct FnSystem<E = FrameEvent>(Box<dyn FnMut(&mut World, &E) + Sync + Send>);
impl<E> FnSystem<E> {
    pub fn new<F>(func: F) -> Self
    where
        F: FnMut(&mut World, &E) + Send + Sync + 'static,
    {
        Self(Box::new(func))
    }
}

impl<E> System<E> for FnSystem<E> {
    fn run(&mut self, world: &mut World, event: &E) {
        self.0(world, event);
    }
}
impl<E> std::fmt::Debug for FnSystem<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FnSystem")
    }
}

enum Label {
    Static(&'static str),
    Dynamic(String),
}
impl Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Label::Static(s) => s,
            Label::Dynamic(s) => s,
        })
    }
}

pub type DynSystem<E = FrameEvent> = Box<dyn System<E> + Send + Sync>;
pub struct SystemGroup<E = FrameEvent>(Label, Vec<DynSystem<E>>);

impl<E> SystemGroup<E> {
    pub fn new(label: &'static str, systems: Vec<DynSystem<E>>) -> Self {
        Self(Label::Static(label), systems)
    }
    pub fn new_with_dynamic_label(label: String, systems: Vec<DynSystem<E>>) -> Self {
        Self(Label::Dynamic(label), systems)
    }
    pub fn add(&mut self, system: DynSystem<E>) -> &mut Self {
        self.1.push(system);
        self
    }
}
impl<E> System<E> for SystemGroup<E> {
    fn run(&mut self, world: &mut World, event: &E) {
        let mut execute = || {
            for system in self.1.iter_mut() {
                // profiling::scope!("sub", format!("iteration {}", i).as_str());
                system.run(world, event);
            }
        };
        match &self.0 {
            Label::Static(s) => {
                profiling::scope!(s);
                execute();
            }
            Label::Dynamic(s) => {
                profiling::scope!("Dynamic", &s);
                execute();
            }
        }
    }
}
impl<E> std::fmt::Debug for SystemGroup<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SystemGroup({}, _)", self.0)
    }
}
